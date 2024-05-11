#![no_std]
#![no_main]

use core::ptr::write_volatile;
use cortex_m::asm::{delay, nop};
use cortex_m_rt::entry;
use nrf52833_pac::Peripherals;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

// const PIN_CNF21= c

fn wait() {
    for _ in 0..400_000 {
        nop();
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let p = Peripherals::take().expect("Failed to take ownership of peripherals");
    p.P0.pin_cnf[21].write(|w| w.dir().output());
    p.P0.pin_cnf[28].write(|w| w.dir().output());

    let mut is_on = false;

    rprintln!("init done, entering loop");
    loop {
        p.P0.out.write(|w| w.pin21().bit(is_on));
        wait();
        is_on = !is_on;
    }
}
