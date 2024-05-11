#![no_std]
#![no_main]

use core::{
    cell::RefCell,
    sync::atomic::{AtomicUsize, Ordering},
};

use defmt::println;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Level, Output, OutputDrive, Pin};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use embassy_sync::{
    blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex},
    mutex::Mutex,
};

type SharedPin = Mutex<ThreadModeRawMutex, Option<Output<'static, AnyPin>>>;
static COL1: SharedPin = Mutex::new(None);
static COL2: SharedPin = Mutex::new(None);
static ROW1: SharedPin = Mutex::new(None);
static ROW2: SharedPin = Mutex::new(None);

// type LedState = Mutex<ThreadModeRawMutex, []

#[embassy_executor::task]
async fn blink1(drive_pin: AnyPin, drain_pin: &'static SharedPin, ms: u64) {
    let mut led = Output::new(drive_pin, Level::Low, OutputDrive::Standard);

    loop {
        led.toggle();
        println!("led: {}", led.is_set_high());
        {
            let mut drain = COL1.lock().await;
            let mut other = COL2.lock().await;
            if let Some(drain) = drain.as_mut() {
                drain.set_low();
            }
            if let Some(other) = other.as_mut() {
                other.set_high();
            }
        }
        Timer::after_millis(ms).await;
    }
}

#[embassy_executor::task]
async fn blink2(drive_pin: AnyPin, drain_pin: &'static SharedPin, ms: u64) {
    let mut led = Output::new(drive_pin, Level::Low, OutputDrive::Standard);

    loop {
        led.toggle();
        // println!("led: {}", led.is_set_high());
        {
            let mut drain = COL2.lock().await;
            let mut other = COL1.lock().await;
            if let Some(drain) = drain.as_mut() {
                drain.set_low();
            }
            if let Some(other) = other.as_mut() {
                other.set_high();
            }
        }
        Timer::after_millis(ms).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    defmt::println!("Hello, World!");

    let col1 = Output::new(p.P0_28.degrade(), Level::Low, OutputDrive::Standard);
    let col2 = Output::new(p.P0_11.degrade(), Level::Low, OutputDrive::Standard);
    // let row1 = Output::new(p.P0_21.degrade(), Level::Low, OutputDrive::Standard);
    let row1 = p.P0_21.degrade();
    // let row2 = Output::new(p.P0_22.degrade(), Level::Low, OutputDrive::Standard);
    let row2 = p.P0_22.degrade();
    *(COL1.lock().await) = Some(col1);
    *(COL2.lock().await) = Some(col2);
    // *(ROW1.lock().await) = Some(row1);
    // *(ROW2.lock().await) = Some(row2);

    spawner.spawn(blink1(row1, &COL1, 500)).unwrap();
    spawner.spawn(blink2(row2, &COL2, 777)).unwrap();

    // spawner
    //     .spawn(blink(p.P0_22.degrade(), p.P0_28.degrade(), 210))
    //     .unwrap();
}
