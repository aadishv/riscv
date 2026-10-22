//! embassy-time driver on the GRTC, using FLPR's domain (SYSCOUNTER[0], INTEN0, GRTC_0 IRQ).
//! The app core's embassy-nrf already starts the GRTC and owns CC[11], so we only read it and use CC[10].

use core::cell::RefCell;

use critical_section::Mutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;
use nrf_pac::clic::vals::{Priority, Trig};
use nrf_pac::grtc::vals::Busy;
use nrf_pac::{GRTC_S as GRTC, Interrupt, VPRCLIC_NS as CLIC};

const DOMAIN: usize = 0;
const CC: usize = 10;

struct GrtcDriver {
    queue: Mutex<RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: GrtcDriver = GrtcDriver {
    queue: Mutex::new(RefCell::new(Queue::new())),
});

impl GrtcDriver {
    fn set_alarm(&self, at: u64) -> bool {
        if at <= self.now() {
            GRTC.intenclr(DOMAIN).write(|w| w.0 = 1 << CC);
            return false;
        }
        GRTC.events_compare(CC).write_value(0);
        // CCL write disables the channel, CCH write re-enables it.
        GRTC.cc(CC).ccl().write_value(at as u32);
        GRTC.cc(CC).cch().write(|w| w.set_cch((at >> 32) as u32 & 0xFFFFF));
        GRTC.intenset(DOMAIN).write(|w| w.0 = 1 << CC);
        true
    }

    fn rearm(&self, queue: &mut Queue) {
        let mut next = queue.next_expiration(self.now());
        while !self.set_alarm(next) {
            next = queue.next_expiration(self.now());
        }
    }
}

impl Driver for GrtcDriver {
    fn now(&self) -> u64 {
        let sc = GRTC.syscounter(DOMAIN);
        sc.active().write(|w| w.set_active(true));
        loop {
            let lo = sc.syscounterl().read();
            let hi = sc.syscounterh().read();
            if hi.busy() == Busy::Ready && !hi.overflow() {
                sc.active().write(|w| w.set_active(false));
                return lo as u64 | (hi.value() as u64) << 32;
            }
        }
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow_ref_mut(cs);
            if queue.schedule_wake(at, waker) {
                self.rearm(&mut queue);
            }
        })
    }
}

/// riscv-rt's `custom-interrupts` hook. On the VPR CLIC, mcause's low 12 bits are the CLIC IRQ number.
#[unsafe(no_mangle)]
fn _dispatch_core_interrupt(code: usize) {
    if code & 0xFFF != Interrupt::GRTC_0 as usize {
        return;
    }
    // Edge-triggered and not hardware-vectored, so the pending bit is ours to clear.
    CLIC.clic().clicint(Interrupt::GRTC_0 as usize).modify(|w| w.set_ip(false));
    if GRTC.events_compare(CC).read() != 0 {
        GRTC.events_compare(CC).write_value(0);
        GRTC.intenclr(DOMAIN).write(|w| w.0 = 1 << CC);
        critical_section::with(|cs| DRIVER.rearm(&mut DRIVER.queue.borrow_ref_mut(cs)));
    }
}

// In CLIC mode, non-vectored traps go to mtvec's base, which must be 64-byte aligned.
core::arch::global_asm!(
    ".section .trap, \"ax\"",
    ".balign 64",
    "_clic_trap:",
    "j _start_trap",
);

pub fn init() {
    unsafe {
        core::arch::asm!("la t0, _clic_trap", "ori t0, t0, 3", "csrw mtvec, t0", out("t0") _);
    }

    GRTC.events_compare(CC).write_value(0);
    GRTC.intenclr(DOMAIN).write(|w| w.0 = 1 << CC);

    CLIC.clic().clicint(Interrupt::GRTC_0 as usize).write(|w| {
        w.set_trig(Trig::EdgeTriggered);
        w.set_priority(Priority::Priolevel1);
        w.set_ie(true);
    });
    unsafe { riscv::interrupt::enable() };
}
