#![no_std]
#![no_main]

use core::ptr::write_volatile;

use cortex_m::asm::nop;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;

// const PIN_CNF21= c

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello, World!");
    loop {}
}
