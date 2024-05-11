#![no_std]
#![no_main]

use core::ptr::write_volatile;
use cortex_m::asm::{delay, nop};
use cortex_m_rt::entry;
use embedded_hal::digital::StatefulOutputPin;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use microbit::{board::Board, hal::timer::Timer};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut board = match Board::take() {
        Some(board) => board,
        None => {
            rprintln!("board/peripherals already taken");
            panic!();
        }
    };
    let mut t0 = Timer::new(board.TIMER0);

    board.display_pins.col1.set_low();
    let mut row1 = board.display_pins.row1;

    rprintln!("init board done, entering loop");
    loop {
        row1.toggle();
        rprintln!("led: {}", row1.is_set_high().unwrap());
        t0.delay_ms(500);
    }
}
