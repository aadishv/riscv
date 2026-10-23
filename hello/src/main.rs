#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::{
    interrupt::{self, InterruptExt},
    pac::{self},
    vpr::Vpr,
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

const FLPR_BIN: &[u8] = include_bytes!("../../flpr2/target/flpr2.bin");
const FLPR_BASE: usize = 0x20067C00;

unsafe extern "C" {
    // Provided by the linker: This is the start of the shared memory buffer
    // used for communicating between the cores.
    pub static mut __shared_mem: u32;
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    info!("hello world");

    unsafe { core::ptr::write_volatile(0x2007_0000 as *mut u16, 6767) };

    let mut vpr = Vpr::new(p.VPR, FLPR_BASE as *const u8).unwrap();
    vpr.load(FLPR_BIN).unwrap();
    cortex_m::asm::dsb();

    interrupt::VPR00.unpend();
    interrupt::VPR00.set_priority(interrupt::Priority::P1);
    unsafe { interrupt::VPR00.enable(); }

    vpr.start();
    pac::VPR00.intenset().write(|v| v.set_triggered20(true));

    loop {
        let value = unsafe { core::ptr::read_volatile(0x2007_0000 as *mut u16) };
        Timer::after_millis(10).await;
        info!("value {}", value);
        if value == 67 {
            break;
        }
    }

    info!("setting task now");

    pac::VPR00.tasks_trigger(21).write_value(1);

    loop {
        let value = unsafe { core::ptr::read_volatile(0x2007_0000 as *mut u16) };
        Timer::after_millis(10).await;
        info!("value {}", value);
        if value == 688 {
            break;
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn VPR00() {
    pac::VPR00.events_triggered(20).write_value(0);
    defmt::info!("flpr event 20");
}
