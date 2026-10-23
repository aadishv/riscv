#![no_std]
#![no_main]
#![feature(abi_riscv_interrupt, fn_align)]

use core::arch::asm;

use nrf_pac::VPRCLIC_NS;
use panic_halt as _;
use riscv_rt::entry;

unsafe extern "C" {
    // Provided by the linker: This is the start of the shared memory buffer
    // used for communicating between the cores.
    pub static mut __shared_mem: u32;
}

#[rustc_align(64)] // mtvec requiemrent apparently
unsafe extern "riscv-interrupt-m" fn spinloop() {
    loop {}
}

unsafe extern "riscv-interrupt-m" fn on_vevif21() {
    vevif::clear_tasks(1 << 21);
    core::ptr::write_volatile(0x2007_0000 as *mut u16, 688);
    vevif::set_events(1 << 20);
}

#[repr(C, align(256))]
struct Vectors([unsafe extern "riscv-interrupt-m" fn(); 64]);

static VECTORS: Vectors = {
    let mut vectors: [unsafe extern "riscv-interrupt-m" fn(); 64] = [spinloop; 64];
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

mod vevif {
    pub unsafe fn set_events(bits: usize) {
        core::arch::asm!(concat!("csrrs x0, ", stringify!(0x7E2), ", {0}"), in(reg)bits);
    }
    pub unsafe fn read_tasks() -> usize {
        let r: usize;
        core::arch::asm!(concat!("csrrs {0}, ", stringify!(0x7E0), ", x0"), out(reg)r);
        r
    }
    pub unsafe fn clear_tasks(bits: usize) -> () {
        core::arch::asm!(concat!("csrrc x0, ", stringify!(0x7E0), ", {0}"), in(reg)bits);
    }
}

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

#[entry]
fn main() -> ! {
    // enable vevif
    let mut ctrl = ctrl::read();
    ctrl.set_enable_rt_periph(true);
    ctrl.set_nordic_key(0x507D);
    unsafe { ctrl::write(ctrl) };

    VPRCLIC_NS.clic().clicint(21).modify(|w| {
        w.set_ie(true);
        w.set_shv(true);
        w.set_priority(nrf_pac::clic::vals::Priority::Priolevel1);
    });
    unsafe { riscv::interrupt::enable() };

    unsafe { core::ptr::write_volatile(0x2007_0000 as *mut u16, 67) };

//     while unsafe { vevif::read_tasks() } & (1 << 21) == 0 {
//         core::hint::spin_loop();
//     }
//
//     unsafe { vevif::clear_tasks(1 << 21) }
//
//     unsafe { core::ptr::write_volatile(0x2007_0000 as *mut u16, 688) };
//
//     unsafe { vevif::set_events(1 << 20) };
    loop {}
}
