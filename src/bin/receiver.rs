use core::slice;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use esp_idf_sys::{ESP_OK, esp_now_recv_info_t, esp_now_register_recv_cb};
use std::sync::{
    OnceLock,
    mpsc::{self, Sender},
};

use esp32_microphone_router::{
    models::{EspNowMessage, Receiver},
    power,
};

static TX_CHANNEL: OnceLock<Sender<EspNowMessage>> = OnceLock::new();

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    power::limit_cpu_speed();

    // --------------------------------------------------------------------------------------------
    // Initialize Wifi.
    // This operation creates several objects that need to live for the entire program.
    // --------------------------------------------------------------------------------------------
    let sys_loop = EspSystemEventLoop::take().expect("Failed to take EspSystemEventLoop.");
    let nvs = EspDefaultNvsPartition::take().expect("Failed to take EspDefaultNvsPartition.");
    let peripherals = Peripherals::take().expect("Failed to take Peripherals.");

    let mut esp_wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))
        .expect("Failed to create EspWifi.");

    let mut wifi =
        BlockingWifi::wrap(&mut esp_wifi, sys_loop).expect("Failed to create BlockingWifi.");

    // Set up ESP32 wifi in STA mode.
    let wifi_config = Configuration::Client(ClientConfiguration::default());

    wifi.set_configuration(&wifi_config)
        .expect("Failed to set wifi configuration in STA mode.");

    wifi.start().expect("Failed to start wifi.");

    // --------------------------------------------------------------------------------------------

    // Create the top-level receiver object.
    let mut receiver = Receiver::new(
        peripherals.pins.gpio23,
        peripherals.pins.gpio22,
        peripherals.pins.gpio21,
        peripherals.pins.gpio19,
    );

    receiver.initialize();

    // Register callback function for whenever a ESP-NOW packet is received.
    if unsafe { esp_now_register_recv_cb(Some(on_data_recv_callback)) } != ESP_OK {
        panic!("esp_now_register_recv_cb() failed.");
    }

    // Set up a channel for the callback function to send, for the main loop to receive.
    let (tx, rx) = mpsc::channel();

    // Move `tx` into the global `TX_CHANNEL` so the callback function can access it.
    if let Err(err) = TX_CHANNEL.set(tx) {
        panic!("Failed to initialize TX_CHANNEL: {:?}", err);
    }

    log::info!("RECEIVER BEGIN");

    // Whenever we receive a message (sent from the callback), let `receiver` handle it.
    loop {
        if let Ok(message) = rx.recv() {
            receiver.update(message);
        }
    }
}

// ============================================================================
// ESP-NOW Receive Callback (Runs inside underlying WiFi Task Context)
// ============================================================================

/// This callback is triggered whenever an ESP-NOW packet arrives, called by a single wifi thread.
/// This function converts packet data into a EspNowMessage and sends it back to main thread for processing.
/// Note: With encryption checked by hardware, we can be confident this is probably a valid packet from our sender.
extern "C" fn on_data_recv_callback(
    _info: *const esp_now_recv_info_t,
    data: *const u8,
    data_len: i32,
) {
    if data.is_null() || data_len <= 0 {
        log::error!("on_data_recv_callback() received invalid data.");
        return;
    }

    let raw_slice = unsafe { slice::from_raw_parts(data, data_len as usize) };

    // Construct a `EspNowMessage` from bytes.
    match postcard::from_bytes::<EspNowMessage>(raw_slice) {
        Err(err) => {
            log::error!("postcard::from_bytes() failed: {}", err);
        }

        Ok(message) => {
            log::info!("ESP-NOW Received message: {:?}", message);

            // Send the message into `TX_CHANNEL`.
            #[allow(clippy::collapsible_if)]
            if let Some(tx) = TX_CHANNEL.get() {
                if let Err(err) = tx.send(message) {
                    log::error!("Failed to forward value to main loop channel: {:?}", err);
                }
            }
        }
    }
}
