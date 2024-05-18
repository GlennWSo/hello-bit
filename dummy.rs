//! This bin is used by crane to build cargo deps for caching
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::println;
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
    println!("dummy");

    loop {}
}
