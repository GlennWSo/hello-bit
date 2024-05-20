#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::println;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_time::Duration;
use microbit_bsp::*;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    println!("entering buzy loop");
}
