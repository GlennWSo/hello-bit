#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::{Duration, Ticker, Timer};
use microbit_bsp::display::Brightness;
use microbit_bsp::embassy_nrf::gpio::{AnyPin, Input};
use microbit_bsp::LedMatrix;
use microbit_bsp::Microbit;

use defmt::Debug2Format as Dbg2;
use defmt::{debug, error, info, println};
use heapless::String;
use ufmt;
use {defmt_rtt as _, panic_probe as _};

use thermostat::{ble::*, pid::PID, ThermoPart, TriThermo, M3, V3};

static ROOM_TEMP: Mutex<ThreadModeRawMutex, f32> = Mutex::new(0.);
static HEAT_POWER: Mutex<ThreadModeRawMutex, f32> = Mutex::new(0.);
static TARGET_TEMP: Mutex<ThreadModeRawMutex, f32> = Mutex::new(20.);

const FFW: f32 = 10.0; // run simulation and ctrl faster

#[embassy_executor::task]
async fn log_globals(mut display: LedMatrix, server: &'static Server) {
    display.set_brightness(Brightness::MAX);
    let mut ble_temp: u8 = 20;
    loop {
        let temp = *ROOM_TEMP.lock().await;
        let target = *TARGET_TEMP.lock().await;
        let heat = *HEAT_POWER.lock().await;
        info!("target:{}, temp: {}, heat:{}", target, temp, heat);

        ble_temp = temp as u8;
        let res = server.bas.battery_level_set(&ble_temp);
        if let Err(e) = res {
            error!("battery set error: {}", e);
            continue;
        };

        let mut msg: String<20> = String::new();
        let decimal = (temp * 10.0) as i32 % 10;
        ufmt::uwriteln!(
            msg,
            " {}.{}C"
            temp as i32,
            decimal
        );
        display.scroll(&msg).await;
    }
}

#[embassy_executor::task]
async fn simple_heater() {
    let dt: u64 = 500;
    let target_temp = 20.;
    let mut ticker = Ticker::every(Duration::from_millis(dt));
    loop {
        {
            let temp = *ROOM_TEMP.lock().await;
            let mut power = HEAT_POWER.lock().await;
            *power = if temp > target_temp { 0. } else { 500. };
        }
        ticker.next().await;
    }
}

type Btn = Input<'static, AnyPin>;
#[embassy_executor::task]
async fn btn_retarget(mut a: Btn, mut b: Btn) {
    loop {
        let diff = match select(a.wait_for_rising_edge(), b.wait_for_rising_edge()).await {
            Either::First(_) => {
                debug!("a rising");
                -2.0
            }
            Either::Second(_) => {
                debug!("b rising");
                3.0
            }
        };
        let mut target = TARGET_TEMP.lock().await;
        *target += diff;
        Timer::after_millis(10).await;
    }
}

#[embassy_executor::task]
async fn pid_heater(p: f32, i: f32, d: f32) {
    let dt: u64 = 100;
    let target_temp = *TARGET_TEMP.lock().await;
    let min_out = 0.;
    let max_out = 500.;
    let mut pid = PID::new(p, i, d, min_out, max_out, target_temp);
    let mut ticker = Ticker::every(Duration::from_millis(dt));
    loop {
        {
            let secs = (dt as f32) / 1000. * FFW;
            let temp = *ROOM_TEMP.lock().await;
            let target_temp = *TARGET_TEMP.lock().await;
            let mut power = HEAT_POWER.lock().await;
            *power = pid.update(target_temp, temp, secs);
        }
        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn simulate_heat(mut model: TriThermo) {
    let dt: u64 = 100;
    let mut ticker = Ticker::every(Duration::from_millis(dt));
    loop {
        {
            let mut room_temp = ROOM_TEMP.lock().await;
            let heat = *HEAT_POWER.lock().await;
            let heat = [heat, 0., 0.];
            let secs = (dt as f32) / 1000. * FFW;
            model.update(secs, heat);
            *room_temp = model.temp()[1];
        }
        ticker.next().await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let board = Microbit::new(config());
    let server = init_ble(spawner).await;

    spawner.spawn(btn_retarget(board.btn_a, board.btn_b));

    let parts = [
        ThermoPart::new(30., Some(10.)),  // heater
        ThermoPart::new(11., Some(100.)), // room
        ThermoPart::new(10., None),       // outside
    ];
    let ab = 0.5; // cond betwen heater and room
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
    spawner.spawn(pid_heater(20., 0.1, -200.)).unwrap();
    spawner.spawn(log_globals(board.display, server)).unwrap();
}
