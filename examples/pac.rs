#![no_std]
#![no_main]
use cortex_m::asm::{nop};
use cortex_m_rt::entry;
use nrf52833_pac::Peripherals;

use defmt::*;
use {defmt_rtt as _, panic_probe as _};

fn wait() {
    for _ in 0..400_000 {
        nop();
    }
}

#[entry]
fn main() -> ! {
    let p = Peripherals::take().expect("Failed to take ownership of peripherals");
    p.P0.pin_cnf[21].write(|w| w.dir().output());
    p.P0.pin_cnf[28].write(|w| w.dir().output());

    let mut is_on = false;

    println!("init done, entering loop");
    loop {
        p.P0.out.write(|w| w.pin21().bit(is_on));
        wait();
        is_on = !is_on;
    }
}
