#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

unsafe extern "C" {
    // Provided by the linker: This is the start of the shared memory buffer
    // used for communicating between the cores.
    pub static mut __shared_mem: u32;
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

#[entry]
fn main() -> ! {
    // enable vevif
    let mut ctrl = flpr2::csr::ctrl::read();
    ctrl.set_enable_rt_periph(true);
    ctrl.set_nordic_key(0x507D);
    unsafe { flpr2::csr::ctrl::write(ctrl) };

    unsafe { core::ptr::write_volatile(0x2007_0000 as *mut u16, 6888) };

    while unsafe { vevif::read_tasks() } & (1 << 21) == 0 {
        core::hint::spin_loop();
    }

    unsafe { vevif::clear_tasks(1 << 21) }

    unsafe { core::ptr::write_volatile(0x2007_0000 as *mut u16, 688) };

    unsafe { vevif::set_events(1 << 20) };
    loop {}
}
