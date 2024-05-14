#![no_std]
#![no_main]

use core::ptr::write_volatile;
use cortex_m::asm::{delay, nop};
use cortex_m_rt::entry;
use embedded_hal::digital::v2::StatefulOutputPin;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use hal::pac;
use nrf52833_hal as hal;

// const PIN_CNF21= c

fn wait() {
    for _ in 0..400_000 {
        nop();
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let p = pac::Peripherals::take().expect("Failed to take ownership of peripherals");
    let port0 = hal::gpio::p0::Parts::new(p.P0);
    let mut row1 = port0.p0_21.into_push_pull_output(hal::gpio::Level::Low);
    let mut _col1 = port0.p0_28.into_push_pull_output(hal::gpio::Level::Low);

    rprintln!("init hal done, entering loop");
    loop {
        row1.toggle();
        rprintln!("led: {}", row1.is_set_high().unwrap());
        wait();
    }
}
