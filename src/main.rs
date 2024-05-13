#![no_std]
#![no_main]

use core::{
    borrow::Borrow,
    cell::RefCell,
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};

use micromath::F32Ext;

use defmt::println;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Level, Output, OutputDrive, Pin};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

use embassy_sync::{
    blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex},
    mutex::Mutex,
};

use microbit_bsp::{
    display::{Brightness, Frame},
    LedMatrix, Microbit,
};

type SharedFrame = Mutex<ThreadModeRawMutex, Option<Frame<5, 5>>>;
static FRAME: SharedFrame = SharedFrame::new(None);

#[embassy_executor::task]
async fn blinker(mut display: LedMatrix, frame: &'static SharedFrame) {
    loop {
        let frame = frame.lock().await.clone();
        display
            .display(frame.unwrap(), Duration::from_millis(1))
            .await;
    }
}

#[embassy_executor::task(pool_size = 25)]
async fn blink(frame: &'static SharedFrame, r: usize, c: usize, ms: u64) {
    let mut is_on = false;
    loop {
        {
            let mut frame = frame.lock().await;
            println!("LED {}:{} is {}", r, c, is_on);
            if let Some(frame) = frame.as_mut() {
                if is_on {
                    frame.set(r, c);
                } else {
                    frame.unset(r, c);
                }
            }
        }
        Timer::after_millis(ms).await;
        is_on = !is_on;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::println!("Hello, World!");
    let board = Microbit::default();
    let mut display = board.display;
    display.set_brightness(Brightness::MAX);
    let mut frame = FRAME.lock().await;
    *frame = Some(Frame::default());

    spawner.spawn(blinker(display, &FRAME)).unwrap();
    let gold: f32 = 1.618033988749;
    for r in 0..5_u32 {
        for c in 0..5_u32 {
            let r: f32 = ((r.pow(2) + c.pow(2)) as f32).sqrt();

            let delay = 100. * gold.powf(r);
            spawner
                .spawn(blink(&FRAME, r as usize, c as usize, delay as u64))
                .unwrap();
        }
    }
}
