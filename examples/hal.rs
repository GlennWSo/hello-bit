#![no_std]
#![no_main]

use cortex_m::asm::{nop};
use cortex_m_rt::entry;
use embedded_hal::digital::StatefulOutputPin;
use hal::pac;
use nrf52833_hal as hal;

use defmt::*;
use {defmt_rtt as _, panic_probe as _};

fn wait() {
    for _ in 0..400_000 {
        nop();
    }
}

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().expect("Failed to take ownership of peripherals");
    let port0 = hal::gpio::p0::Parts::new(p.P0);
    let mut row1 = port0.p0_21.into_push_pull_output(hal::gpio::Level::Low);
    let mut _col1 = port0.p0_28.into_push_pull_output(hal::gpio::Level::Low);

    println!("init hal done, entering loop");
    loop {
        row1.toggle();
        println!("led: {}", row1.is_set_high().unwrap());
        wait();
    }
}
