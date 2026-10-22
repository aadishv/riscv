#![no_std]
#![no_main]

mod time_driver;

use embassy_executor::Spawner;
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    time_driver::init();
    Timer::after_secs(5).await;
    unsafe { core::ptr::write_volatile(0x2007_0000 as *mut u16, 688) };
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
