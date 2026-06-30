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
        OnceLock,
        mpsc::{self, Sender},
    },
};

use esp32_microphone_router::{
    esp_now,
    models::{
        ESP_NOW_MESSAGE_SIZE, EspNowMessage, EspNowMessageHeader, MicrophoneId, MicrophoneRoute,
        ReceiverState, ResetMicrophonePayload, UpdateRoutableMicrophonePayload,
        UpdateSimpleMicrophonePayload,
    },
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

    esp_now::initialize_esp_now_as_receiver();

    if unsafe { esp_now_register_recv_cb(Some(on_data_recv_callback)) } != ESP_OK {
        panic!("Failed to register receive callback.");
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

    if let Err(err) = TX_CHANNEL.set(tx) {
        panic!("Failed to initialize TX_CHANNEL: {:?}", err);
    }

    let mut state = ReceiverState::default();

    log::info!("RECEIVER BEGIN");

    // Whenever we receive a message, if it is valid, process it to update `state`, and then flush `state` to relays.
    loop {
        if let Ok(message) = rx.recv() {
            process_message(&mut state, message);
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

/// Look into the union and pass the payload to the appropriate handler.
fn process_message(state: &mut ReceiverState, message: EspNowMessage) {
    log::info!("process_message: {:?}", message);

    match message.header {
        EspNowMessageHeader::ResetMicrophone => {
            process_reset_microphone(state, unsafe { message.payload.reset_microphone })
        }
        EspNowMessageHeader::UpdateRoutableMicrophone => {
            process_update_routable_microphone(state, unsafe {
                message.payload.update_routable_microphone
            })
        }
        EspNowMessageHeader::UpdateSimpleMicrophone => {
            process_update_simple_microphone(state, unsafe {
                message.payload.update_simple_microphone
            })
        }
    };
}

/// Reset the specific microphone's `last_message_id`.
fn process_reset_microphone(state: &mut ReceiverState, payload: ResetMicrophonePayload) {
    match payload.microphone_id {
        MicrophoneId::RoutableMicrophone => {
            state.routable_microphone.last_message_id = 0;
        }
        MicrophoneId::SimpleMicrophone => {
            state.simple_microphone.last_message_id = 0;
        }
    }
}

/// If `message_id` is valid, update the state of the routable microphone.
fn process_update_routable_microphone(
    state: &mut ReceiverState,
    payload: UpdateRoutableMicrophonePayload,
) {
    if payload.message_id > state.routable_microphone.last_message_id {
        state.routable_microphone.route = payload.route;
        state.routable_microphone.active = payload.active;
        state.routable_microphone.last_message_id = payload.message_id;
        log::info!("Accepted this message.");
    } else {
        log::warn!("Rejected this old message.");
    }
}

/// If `message_id` is valid, update the state of the simple microphone.
fn process_update_simple_microphone(
    state: &mut ReceiverState,
    payload: UpdateSimpleMicrophonePayload,
) {
    if payload.message_id > state.simple_microphone.last_message_id {
        state.simple_microphone.active = payload.active;
        state.simple_microphone.last_message_id = payload.message_id;
        log::info!("Accepted this message.");
    } else {
        log::warn!("Rejected this old message.");
    }
}

/// Update the physical operation of the relays to match the `state`.
fn flush_state<'a>(
    state: &ReceiverState,
    relay_muting_routable_microphone: &mut PinDriver<'a, Output>,
    relay_routing1: &mut PinDriver<'a, Output>,
    relay_routing2: &mut PinDriver<'a, Output>,
    relay_muting_simple_microphone: &mut PinDriver<'a, Output>,
) {
    // Mute or unmute Routable Microphone.
    if state.routable_microphone.active {
        relay_muting_routable_microphone.set_high().unwrap();
    } else {
        relay_muting_routable_microphone.set_low().unwrap();
    }

    // Update routing of Routable Microphone.
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

    // Mute or unmute Simple Microphone.
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

    if data_len != ESP_NOW_MESSAGE_SIZE {
        log::error!(
            "Unexpected payload size! Expected {} bytes, received {} bytes.",
            ESP_NOW_MESSAGE_SIZE,
            data_len
        );
        return;
    }

    let message = unsafe { ptr::read_unaligned(data as *const EspNowMessage) };

    log::info!("ESP-NOW Received message: {:?}", message);

    #[allow(clippy::collapsible_if)]
    if let Some(tx) = TX_CHANNEL.get() {
        if let Err(err) = tx.send(message) {
            log::error!("Failed to forward value to main loop channel: {:?}", err);
        }
    }
}
