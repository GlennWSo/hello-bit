#![no_std]
#![no_main]

use core::sync::atomic::{AtomicUsize, Ordering};

use defmt::println;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Level, Output, OutputDrive, Pin};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn blink(drive_pin: AnyPin, drain_pin: AnyPin, ms: u64) {
    let mut led = Output::new(drive_pin, Level::Low, OutputDrive::Standard);
    let _col1 = Output::new(drain_pin, Level::Low, OutputDrive::Standard);

    loop {
        led.toggle();
        println!("led: {}", led.is_set_high());
        Timer::after_millis(ms).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    defmt::println!("Hello, World!");

    // let _col1 = Output::new(p.P0_28, Level::Low, OutputDrive::Standard);
    spawner
        .spawn(blink(p.P0_21.degrade(), p.P0_28.degrade(), 500))
        .unwrap();
}
