#![no_std]
#![no_main]
#![feature(abi_riscv_interrupt, fn_align)]

use panic_halt as _;
use riscv_rt::entry;

mod ipc;

#[entry]
fn main() -> ! {
    ipc::init();

    loop {}
}
