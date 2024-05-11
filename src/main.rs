#![no_std]
#![no_main]

use core::sync::atomic::{AtomicUsize, Ordering};

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
// use panic_halt as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let _col1 = Output::new(p.P0_28, Level::Low, OutputDrive::Standard);
    let mut led = Output::new(p.P0_21, Level::Low, OutputDrive::Standard);

    defmt::println!("Hello, World!");
    loop {
        led.set_high();
        defmt::println!("led: {}", led.is_set_high());
        Timer::after_millis(300).await;
        led.set_low();
        Timer::after_millis(300).await;
    }
}
