#![no_std]
#![no_main]

use core::ptr::write_volatile;
use cortex_m::asm::{delay, nop};
use cortex_m_rt::entry;
use defmt::*;
use embedded_hal::digital::StatefulOutputPin;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use microbit::{board::Board, hal::timer::Timer};
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
    let mut board = match Board::take() {
        Some(board) => board,
        None => {
            defmt::panic!("board/peripherals already taken");
        }
    };
    let mut t0 = Timer::new(board.TIMER0);

    board.display_pins.col1.set_low();
    let mut row1 = board.display_pins.row1;

    println!("init board done, entering loop");
    loop {
        row1.toggle();
        println!("led: {}", row1.is_set_high().unwrap());
        t0.delay_ms(500);
    }
}
