use core::sync::atomic::{AtomicBool, Ordering};

use defmt::{info, warn};
use embassy_nrf::{Peri, interrupt::{self, InterruptExt}, pac, peripherals::VPR, vpr::Vpr};
use rpc::CpuToFlpr;

use crate::flpr::shared_mem::flpr_to_cpu_buffer;

const FLPR_BASE: usize = 0x20067C00;

mod shared_mem {
    use rpc::{CpuToFlpr, FlprToCpu};

    unsafe extern "C" {
        // from linker
        // TODO: remove pub when done debugging
        pub static mut __shared_mem: u32;
    }

    pub fn cpu_to_flpr_buffer() -> *mut CpuToFlpr {
        return core::ptr::addr_of!(__shared_mem) as _;
    }
    pub fn flpr_to_cpu_buffer() -> *mut FlprToCpu {
        return unsafe { core::ptr::addr_of!(__shared_mem).add(4096/2) as _ };
    }
}

pub fn start(bin: &[u8], vpr: Peri<'static, VPR>) {
    warn!("{:x}", unsafe { shared_mem::cpu_to_flpr_buffer() as usize });
    let mut vpr = Vpr::new(vpr, FLPR_BASE as *const u8).unwrap();
    vpr.load(bin).unwrap();
    cortex_m::asm::dsb();

    interrupt::VPR00.unpend();
    interrupt::VPR00.set_priority(interrupt::Priority::P1);
    unsafe { interrupt::VPR00.enable(); }

    vpr.start();
    pac::VPR00.intenset().write(|v| {
        v.set_triggered20(true);
        v.set_triggered21(true);
    });
}

pub fn send(message: CpuToFlpr) -> Result<(), ()> {
    if TRANSMISSION_IN_PROGRESS.load(Ordering::Acquire) {
        return Err(());
    }
    unsafe { *shared_mem::cpu_to_flpr_buffer() = message; }
    pac::VPR00.tasks_trigger(20).write_value(1);
    TRANSMISSION_IN_PROGRESS.store(true, Ordering::Release);
    Ok(())
}

static TRANSMISSION_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

#[unsafe(no_mangle)]
extern "C" fn VPR00() {
    if pac::VPR00.events_triggered(20).read() != 0 {
        pac::VPR00.events_triggered(20).write_value(0);
        // TODO: handle new message
        info!("got new message!!!!! {}", unsafe { *flpr_to_cpu_buffer() });
        pac::VPR00.tasks_trigger(21).write_value(1);
    }
    if pac::VPR00.events_triggered(21).read() != 0 {
        info!("flpr marked read");
        TRANSMISSION_IN_PROGRESS.store(false, Ordering::Release);
        pac::VPR00.events_triggered(21).write_value(0);
    }
}




/*
use core::{arch::asm, sync::atomic::{AtomicBool, Ordering}};

use nrf_pac::{VPRCLIC_NS, shared};
use rpc::FlprToCpu;

unsafe extern "C" {
    // from linker
    static mut __shared_mem: u32;
}

mod shared_mem {
    use rpc::{CpuToFlpr, FlprToCpu};
    use crate::flpr::__shared_mem;

    pub fn cpu_to_flpr_buffer() -> *mut CpuToFlpr {
        return unsafe { __shared_mem as _ };
    }
    pub fn flpr_to_cpu_buffer() -> *mut FlprToCpu {
        return unsafe { (__shared_mem+4096/2) as _ };
    }
}

static TRANSMISSION_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

mod ctrl {
    use riscv::{read_write_csr, read_write_csr_field};

    read_write_csr! {
        /// Nordic Control Register
        NordicCtrl: 0x7C0,
        mask: usize::MAX,
    }

    read_write_csr_field! {
        NordicCtrl,
        /// Control bit to enable Real-Time Peripherals
        enable_rt_periph: 0,
    }

    read_write_csr_field! {
        NordicCtrl,
        /// Arbitration priority on bus
        nordic_key: [16:31],
    }
}

pub unsafe fn set_events(bits: usize) {
    core::arch::asm!(concat!("csrrs x0, ", stringify!(0x7E2), ", {0}"), in(reg)bits);
}

pub unsafe fn clear_tasks(bits: usize) -> () {
    core::arch::asm!(concat!("csrrc x0, ", stringify!(0x7E0), ", {0}"), in(reg)bits);
}

#[rustc_align(64)] // mtvec requiemrent apparently
unsafe extern "riscv-interrupt-m" fn spinloop() {
    loop {}
}

unsafe extern "riscv-interrupt-m" fn on_vevif20() {
    clear_tasks(1 << 20);
    // TODO: handle incoming message
    let _ = send(FlprToCpu::Pong);
    set_events(1 << 21);
}

unsafe extern "riscv-interrupt-m" fn on_vevif21() {
    clear_tasks(1 << 21);
    TRANSMISSION_IN_PROGRESS.store(false, Ordering::Release);
}

#[repr(C, align(256))]
struct Vectors([unsafe extern "riscv-interrupt-m" fn(); 64]);

static VECTORS: Vectors = {
    let mut vectors: [unsafe extern "riscv-interrupt-m" fn(); 64] = [spinloop; 64];
    vectors[16 + 20] = on_vevif20;
    vectors[16 + 21] = on_vevif21;
    Vectors(vectors)
};

#[riscv_rt::setup_interrupts]
fn setup_interrupts() {
    unsafe {
        asm!("csrw mtvec, {0}", in(reg) spinloop as *const () as usize | 0b11); // exception
        asm!("csrw 0x307, {0}", in(reg) &VECTORS as *const _ as usize); // mtvt
    }
}

pub fn init() {
    // enable vevif
    let mut ctrl = ctrl::read();
    ctrl.set_enable_rt_periph(true);
    ctrl.set_nordic_key(0x507D);
    unsafe { ctrl::write(ctrl) };

    VPRCLIC_NS.clic().clicint(20).modify(|w| {
        w.set_ie(true);
        w.set_shv(true);
        w.set_priority(nrf_pac::clic::vals::Priority::Priolevel1);
    });
    VPRCLIC_NS.clic().clicint(21).modify(|w| {
        w.set_ie(true);
        w.set_shv(true);
        w.set_priority(nrf_pac::clic::vals::Priority::Priolevel1);
    });
    unsafe { riscv::interrupt::enable() };
}
pub fn send(message: FlprToCpu) -> Result<(), ()> {
    if TRANSMISSION_IN_PROGRESS.load(Ordering::Acquire) {
        return Err(());
    }
    unsafe { *shared_mem::flpr_to_cpu_buffer() = message; }
    TRANSMISSION_IN_PROGRESS.store(true, Ordering::Release);
    Ok(())
}
*/