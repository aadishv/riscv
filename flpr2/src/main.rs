#![no_std]
#![no_main]
#![feature(abi_riscv_interrupt, fn_align)]

mod ipc;

use core::{arch::asm, cell::RefCell, sync::atomic::AtomicBool};

use nrf_pac::VPRCLIC_NS;
use panic_halt as _;
use riscv_rt::entry;


#[entry]
fn main() -> ! {
    ipc::init();

    loop {}
}
