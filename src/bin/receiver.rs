use esp_idf_hal::{
    gpio::{Output, PinDriver},
    peripherals::Peripherals,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use esp_idf_sys::{ESP_OK, esp_now_recv_info_t, esp_now_register_recv_cb};
use std::{
    ptr,
    sync::{
        Mutex, OnceLock,
        mpsc::{self, Sender},
    },
};

use esp32_microphone_router::{
    esp_now,
    models::{MESSAGE_SIZE, Message, MicrophoneRoute, ReceiverState},
};

static TX_CHANNEL: OnceLock<Mutex<Sender<Message>>> = OnceLock::new();

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

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

    esp_now::initialize_esp_now_as_receiver();

    let cb_err = unsafe { esp_now_register_recv_cb(Some(on_data_recv_callback)) };
    if cb_err != ESP_OK {
        panic!("Failed to register receive callback: {:?}", cb_err);
    }

    // ---------------------------------------------------------------------------------------
    // Set up relays.
    // ---------------------------------------------------------------------------------------
    let mut relay_muting_routable_microphone = PinDriver::output(peripherals.pins.gpio23).unwrap();
    let mut relay_routing1 = PinDriver::output(peripherals.pins.gpio22).unwrap();
    let mut relay_routing2 = PinDriver::output(peripherals.pins.gpio21).unwrap();
    let mut relay_muting_simple_microphone = PinDriver::output(peripherals.pins.gpio19).unwrap();

    relay_muting_routable_microphone.set_low().unwrap();
    relay_routing1.set_low().unwrap();
    relay_routing2.set_low().unwrap();
    relay_muting_simple_microphone.set_low().unwrap();

    // ---------------------------------------------------------------------------------------
    // Set up a channel for the callback function to send, for the main loop to receive.
    // ---------------------------------------------------------------------------------------
    let (tx, rx) = mpsc::channel();

    if let Err(err) = TX_CHANNEL.set(Mutex::new(tx)) {
        panic!("Failed to initialize TX_CHANNEL: {:?}", err);
    }

    let mut state = ReceiverState::default();

    log::info!("\n\n RECEIVER version 1.0 \n\n");

    // Whenever we receive a message, if it is valid, process it to update state, and then flush state to relays.
    loop {
        if let Ok(message) = rx.recv()
            && process_message(&mut state, message)
        {
            flush_state(
                &state,
                &mut relay_muting_routable_microphone,
                &mut relay_routing1,
                &mut relay_routing2,
                &mut relay_muting_simple_microphone,
            );
        }
    }
}

/// Update state from the message.
/// Return a bool indicating if the message was accepted (or ignored because its timestamp is too old).
fn process_message(state: &mut ReceiverState, message: Message) -> bool {
    log::info!("\nprocess_message: {:?}", message);

    match message.route {
        Some(route) => {
            if message.message_id > state.routable_microphone.last_message_id {
                state.routable_microphone.route = route;
                state.routable_microphone.active = message.active;
                state.routable_microphone.last_message_id = message.message_id;
                log::warn!("Accepted this message.");
                true
            } else {
                log::warn!("Rejected this old message.");
                false
            }
        }
        None => {
            if message.message_id > state.simple_microphone.last_message_id {
                state.simple_microphone.active = message.active;
                state.simple_microphone.last_message_id = message.message_id;
                log::warn!("Accepted this message.");
                true
            } else {
                log::warn!("Rejected this old message.");
                false
            }
        }
    }
}

fn flush_state<'a>(
    state: &ReceiverState,
    relay_muting_routable_microphone: &mut PinDriver<'a, Output>,
    relay_routing1: &mut PinDriver<'a, Output>,
    relay_routing2: &mut PinDriver<'a, Output>,
    relay_muting_simple_microphone: &mut PinDriver<'a, Output>,
) {
    if state.routable_microphone.active {
        relay_muting_routable_microphone.set_high().unwrap();
    } else {
        relay_muting_routable_microphone.set_low().unwrap();
    }

    match state.routable_microphone.route {
        MicrophoneRoute::ToAudience => {
            relay_routing1.set_low().unwrap();
            relay_routing2.set_low().unwrap();
        }
        MicrophoneRoute::ToBand => {
            relay_routing1.set_high().unwrap();
            relay_routing2.set_high().unwrap();
        }
    }

    if state.simple_microphone.active {
        relay_muting_simple_microphone.set_high().unwrap();
    } else {
        relay_muting_simple_microphone.set_low().unwrap();
    }
}

// ============================================================================
// ESP-NOW Receive Callback (Runs inside underlying WiFi Task Context)
// ============================================================================

/// Callback triggered automatically whenever an ESP-NOW packet arrives.
/// This function converts data into a Message and sends it back to main for processing.
/// With encryption checked by hardware, we can be sure this message really came from our sender.
extern "C" fn on_data_recv_callback(
    _info: *const esp_now_recv_info_t,
    data: *const u8,
    data_len: i32,
) {
    let data_len = data_len as usize;

    if data_len != MESSAGE_SIZE {
        log::error!(
            "Unexpected payload size! Expected {} bytes, received {} bytes.",
            MESSAGE_SIZE,
            data_len
        );
        return;
    }

    let message = unsafe { ptr::read_unaligned(data as *const Message) };

    log::info!("ESP-NOW Received message: {:?}", message);

    if let Some(mutex) = TX_CHANNEL.get()
        && let Ok(tx) = mutex.lock()
    {
        if let Err(e) = tx.send(message) {
            log::error!("Failed to forward value to main loop channel: {:?}", e);
        }
    }
}
