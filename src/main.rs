#![no_std]
#![no_main]

use micromath::F32Ext;

use defmt::{info, println};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};

use microbit_bsp::{
    display::{Brightness, Frame},
    LedMatrix, Microbit,
};

type SharedFrame = Mutex<ThreadModeRawMutex, Option<Frame<5, 5>>>;
static FRAME: SharedFrame = SharedFrame::new(None);

#[embassy_executor::task]
async fn blinker(mut display: LedMatrix, frame: &'static SharedFrame) {
    loop {
        let frame = *frame.lock().await;
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
            info!("LED {}:{} is {}", r, c, is_on);
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
    let gold: f32 = 1.618_034;
    for r in 0..5_i32 {
        for c in 0..5_i32 {
            let c_dist = (2 - c);
            let r_dist = (2 - r);
            let radi: f32 = ((r_dist.pow(2) + c_dist.pow(2)) as f32).sqrt();

            let rc_part = (c as f32) * 10. * gold + (r as f32) * 20. * gold;
            let delay = 100. * gold.powf(radi) + rc_part;
            spawner
                .spawn(blink(&FRAME, r as usize, c as usize, delay as u64))
                .unwrap();
        }
    }
}
