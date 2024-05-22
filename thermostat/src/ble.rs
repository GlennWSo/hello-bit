use nrf_softdevice::ble::{gatt_server, peripheral, Connection};
use nrf_softdevice::{raw, Softdevice};

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use microbit_bsp::{Config, Priority};

use heapless::Vec;
use static_cell::StaticCell;

use defmt::{debug, info};

static CONN: Mutex<ThreadModeRawMutex, Option<Connection>> = Mutex::new(None);

#[nrf_softdevice::gatt_server]
pub struct Server {
    bas: BatteryService,
}
static SERVER: StaticCell<Server> = StaticCell::new();

#[nrf_softdevice::gatt_service(uuid = "180f")]
pub struct BatteryService {
    #[characteristic(uuid = "2a19", read, notify)]
    battery_level: u8,
}

// Application must run at a lower priority than softdevice
pub fn config() -> Config {
    let mut config = Config::default();
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    config
}

#[embassy_executor::task(pool_size = "1")]
pub async fn gatt_server_task(server: &'static Server) {
    {
        let conn = {
            let lock = CONN.lock().await;
            lock.as_ref().unwrap().clone() // clone is used here so we can drop the lock
        };

        gatt_server::run(&conn, server, |e| match e {
            ServerEvent::Bas(e) => match e {
                BatteryServiceEvent::BatteryLevelCccdWrite { notifications } => {
                    info!("battery notifications: {}", notifications);
                }
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
    let service_list_16 = [
        3, // the len - 1
        raw::BLE_GAP_AD_TYPE_16BIT_SERVICE_UUID_COMPLETE as u8,
        0x018, // part of 0x180F which u16 UIID for battery service
        0x0F,  // part of 0x180F which u16 UIID for battery service
    ];
    adv_data.extend(service_list_16.into_iter());

    adv_data
        .extend_from_slice(&[
            (1 + name.len() as u8),
            raw::BLE_GAP_AD_TYPE_COMPLETE_LOCAL_NAME as u8,
        ])
        .unwrap();

    adv_data.extend_from_slice(name.as_bytes()).ok().unwrap();

    // TODO: refer to some docs here to explain magic values
    #[rustfmt::skip]
    let scan_data = &[
        0x03, 0x03, 0x18, 0x0F
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

fn enable_softdevice(name: &'static str) -> &'static mut Softdevice {
    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 4,
            rc_temp_ctiv: 2,
            accuracy: 7,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 2,
            event_length: 24,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 128 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: 32768,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 3,
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: name.as_ptr() as *const u8 as _,
            current_len: name.len() as u16,
            max_len: name.len() as u16,
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
