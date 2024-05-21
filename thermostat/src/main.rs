#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::{info, println};
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
use microbit_bsp::*;
use {defmt_rtt as _, panic_probe as _};

use defmt::Debug2Format as Dbg2;
use thermostat::{ThermoPart, TriThermo, M3, V3};

static ROOM_TEMP: Mutex<ThreadModeRawMutex, f32> = Mutex::new(0.);

#[embassy_executor::task]
async fn simulate_heat(mut model: TriThermo) {
    let dt: u64 = 10;
    loop {
        model.diffuse((dt as f32) / 1000.);
        Timer::after_millis(dt).await;
        info!("temp: {:?}", Dbg2(&model.temp()));
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let board = Microbit::default();

    let parts = [
        ThermoPart::new(30., Some(10.)),  // heater
        ThermoPart::new(11., Some(100.)), // room
        ThermoPart::new(10., None),       // outside
    ];
    let ab = 1.0; // cond betwen heater and room
    let bc = 1.0; // cond betwen room and outside
    let mut connections: M3 = [
        [0., ab, 0.], // connect heater
        [ab, 0., bc],
        [0., bc, 0.],
    ]
    .into();
    let column: V3 = [1., 1., 1.].into();
    let diag = M3::from_diagonal(&(connections * column));
    connections -= diag;
    println!("diag: {}", Dbg2(&diag));
    let mut model = TriThermo::new(parts.into(), connections.into());
    println!("ThermoDyn system: {:#?}", Dbg2(&model));
    spawner.spawn(simulate_heat(model)).unwrap();
}
