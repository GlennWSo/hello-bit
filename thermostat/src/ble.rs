use nrf_softdevice::ble::{gatt_server, peripheral, Connection};
use nrf_softdevice::{raw, Softdevice};

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use microbit_bsp::{Config, Priority};

use heapless::Vec;
use static_cell::StaticCell;

use defmt::{debug, error, info};

pub static CONN: Mutex<ThreadModeRawMutex, Option<Connection>> = Mutex::new(None);
pub static TARGET_TEMP: Mutex<ThreadModeRawMutex, f32> = Mutex::new(20.);

#[nrf_softdevice::gatt_server]
pub struct Server {
    pub thermo: ThermoService,
}
static SERVER: StaticCell<Server> = StaticCell::new();

#[nrf_softdevice::gatt_service(uuid = "180f")]
pub struct ThermoService {
    #[characteristic(uuid = "2a6e", read, notify)]
    pub current_temprature: i32,
    #[characteristic(uuid = "2a6e", read, write)]
    pub target_temprature: i32,
}

// Application must run at a lower priority than softdevice
pub fn config() -> Config {
    let mut config = Config::default();
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    config
}

const MAX_CONN: u8 = 2;

#[embassy_executor::task(pool_size = MAX_CONN as usize)]
pub async fn gatt_server_task(server: &'static Server) {
    {
        let conn = {
            let lock = CONN.lock().await;
            lock.as_ref().unwrap().clone() // clone is used here so we can drop the lock
        };

        gatt_server::run(&conn, server, |e| match e {
            ServerEvent::Thermo(e) => match e {
                ThermoServiceEvent::CurrentTempratureCccdWrite { notifications } => {
                    info!("battery notifications: {}", notifications);
                }
                ThermoServiceEvent::TargetTempratureWrite(v) => match TARGET_TEMP.try_lock() {
                    Ok(mut target) => *target = v as f32 / 100.,
                    Err(err) => error!("failed write to TARGET_TEMP: {}", err),
                },
            },
        })
        .await;
        info!("connection closed");
    }
    let mut lock = CONN.lock().await;
    lock.take();
}

#[embassy_executor::task]
pub async fn advertiser_task(
    spawner: Spawner,
    sd: &'static Softdevice,
    server: &'static Server,
    name: &'static str,
) {
    // spec for assigned numbers: https://www.bluetooth.com/wp-content/uploads/Files/Specification/HTML/Assigned_Numbers/out/en/Assigned_Numbers.pdf?v=1715770644767
    let mut adv_data: Vec<u8, 31> = Vec::new();
    let flags: [u8; 3] = [
        2, //the len -1
        raw::BLE_GAP_AD_TYPE_FLAGS as u8,
        raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
    ];
    adv_data.extend(flags.into_iter());
    #[rustfmt::skip]
    let advertized_services = [
        3, // len - 1
        raw::BLE_GAP_AD_TYPE_16BIT_SERVICE_UUID_COMPLETE as u8,
        0x09, 0x018, // u16 UIID for health-thermometer service
    ];
    adv_data.extend(advertized_services.into_iter());

    adv_data
        .extend_from_slice(&[
            (1 + name.len() as u8),
            raw::BLE_GAP_AD_TYPE_COMPLETE_LOCAL_NAME as u8,
        ])
        .unwrap();

    adv_data.extend_from_slice(name.as_bytes()).ok().unwrap();

    // additional services shown when scanned
    #[rustfmt::skip]
    let scan_data = &[
        // 0x03, // len -1 
        // raw::BLE_GAP_AD_TYPE_16BIT_SERVICE_UUID_COMPLETE as u8,
        // 0x09, 0x018, // u16 UIID for health-thermometer service
    ];

    loop {
        let config = peripheral::Config::default();
        let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
            adv_data: &adv_data[..],
            scan_data,
        };
        debug!("advertising");
        let conn = peripheral::advertise_connectable(sd, adv, &config)
            .await
            .unwrap();

        defmt::debug!("connection established");
        let mut lock = CONN.lock().await;
        lock.replace(conn);

        if let Err(e) = spawner.spawn(gatt_server_task(server)) {
            defmt::warn!("Error spawning gatt task: {:?}", e);
        }
    }
}

fn enable_softdevice(gap_name: &'static str) -> &'static mut Softdevice {
    let config = nrf_softdevice::Config {
        /// low frequency clock config
        clock: Some(raw::nrf_clock_lf_cfg_t {
            /// [nrf docs](https://infocenter.nordicsemi.com/topic/ps_nrf52833/clock.html?cp=5_1_0_4_3_2_20#register.LFCLKSRC)
            /// the clock source type to use
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 4, // calibiration timer in seconds/4
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_20_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: MAX_CONN,
            event_length: 24,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 128 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: 32768,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: MAX_CONN,
            // periph_role_count: 3,
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: gap_name.as_ptr() as *const u8 as _,
            current_len: gap_name.len() as u16,
            max_len: gap_name.len() as u16,
            write_perm: unsafe { core::mem::zeroed() },
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                raw::BLE_GATTS_VLOC_STACK as u8,
            ),
        }),
        ..Default::default()
    };
    Softdevice::enable(&config)
}

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}

pub async fn init_ble(s: Spawner) -> &'static Server {
    // Spawn the underlying softdevice task
    let device_name = "Embassy Microbit";
    let sd = enable_softdevice(device_name);

    // Create a BLE GATT server and make it static
    // let server =
    let server = SERVER.init(Server::new(sd).unwrap());

    // server.bas.battery_level_set(&13).unwrap();
    s.spawn(softdevice_task(sd)).unwrap();
    // Starts the bluetooth advertisement and GATT server
    s.spawn(advertiser_task(s, sd, server, device_name))
        .unwrap();
    server
}
