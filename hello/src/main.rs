#![no_std]
#![no_main]

use defmt::{dbg, info};
use embassy_executor::Spawner;
use embassy_nrf::{
    interrupt::{self, InterruptExt},
    pac::{self},
    vpr::Vpr,
};
use embassy_time::Timer;
use rpc::CpuToFlpr;
use {defmt_rtt as _, panic_probe as _};

mod flpr;

const FLPR_BIN: &[u8] = include_bytes!("../../flpr2/target/flpr2.bin");

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    unsafe { core::ptr::write_volatile(0x2007_0000 as *mut u32, 0) };

    let p = embassy_nrf::init(Default::default());

    info!("stat {}", unsafe { core::ptr::read_volatile(0x2007_0000 as *mut u32) });

    info!("hello world");
    flpr::start(FLPR_BIN, p.VPR);

    Timer::after_millis(100).await;
    info!("heyo");
    let _ = flpr::send(CpuToFlpr::Ping);
    loop {
        info!("stat {}", unsafe { core::ptr::read_volatile(0x2007_0000 as *mut u32) });
        Timer::after_millis(100).await;
    }

//     unsafe { core::ptr::write_volatile(0x2007_0000 as *mut u32, 6767) };
//
//     loop {
//         let value = unsafe { core::ptr::read_volatile(0x2007_0000 as *mut u32) };
//         Timer::after_millis(10).await;
//         info!("value {}", value);
//         if value == 67 {
//             break;
//         }
//     }
//
//     info!("setting task now");
//
//     pac::VPR00.tasks_trigger(21).write_value(1);
//
//     loop {
//         let value = unsafe { core::ptr::read_volatile(0x2007_0000 as *mut u32) };
//         Timer::after_millis(10).await;
//         info!("value {}", value);
//         if value == 688 {
//             break;
//         }
//     }
}
