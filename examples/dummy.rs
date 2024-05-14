//! This bin is used by crane to build cargo deps for caching
#![no_std]
#![no_main]

use cortex_m::asm::{delay, nop};
use cortex_m_rt::entry;
use defmt::println;
use nrf52833_pac::Peripherals;
use {defmt_rtt as _, panic_probe as _};

fn wait() {
    for _ in 0..400_000 {
        nop();
    }
}

#[entry]
fn main() -> ! {
    println!("hi");
    let p = Peripherals::take().expect("Failed to take ownership of peripherals");
    p.P0.pin_cnf[21].write(|w| w.dir().output());
    p.P0.pin_cnf[28].write(|w| w.dir().output());

    let mut is_on = false;

    loop {
        p.P0.out.write(|w| w.pin21().bit(is_on));
        wait();
        is_on = !is_on;
    }
}
