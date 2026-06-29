use edge_executor::LocalExecutor;
use esp_idf_hal::{
    gpio::{Level, PinDriver},
    peripherals::Peripherals,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use esp_idf_sys::{ESP_OK, esp_now_send};
use std::{cell::RefCell, rc::Rc};

use esp32_microphone_router::{
    button::Button,
    config::RECEIVER_MAC,
    esp_now,
    models::{
        MESSAGE_SIZE, Message, RoutableMicrophoneSenderState, SimpleMicrophoneSenderState,
        ToMessage,
    },
};

fn main() {
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

    esp_now::initialize_esp_now_as_sender();

    // --------------------------------------------------------------------------------------------

    // --------------------------------------------------------------------------------------------

    // Set up input devices with callbacks.
    let state = Rc::new(RefCell::new(RoutableMicrophoneSenderState::default()));

    let mut latch_led = PinDriver::output(peripherals.pins.gpio23).expect("Failed to set up LED.");
    latch_led.set_low().unwrap();

    let state1 = Rc::clone(&state);
    let jimmy_latch = Button::new(
        peripherals.pins.gpio22,
        Box::new(move || {
            log::info!("jimmy_latch PRESS");

            // 1. Flip the latch state.
            let mut _state = state1.borrow_mut();
            _state.to_audience_latch_is_pressed = !_state.to_audience_latch_is_pressed;

            // 2. Update the latch led.
            let level = if _state.to_audience_latch_is_pressed {
                Level::High
            } else {
                Level::Low
            };
            latch_led.set_level(level).unwrap();

            // 3. Send message.
            send_message(_state.to_message());
        }),
        Box::new(move || {
            log::info!("jimmy_latch RELEASE (do nothing)");
        }),
    );

    let (state1, state2) = (Rc::clone(&state), Rc::clone(&state));
    let jimmy_pushbutton = Button::new(
        peripherals.pins.gpio21,
        Box::new(move || {
            log::info!("jimmy_pushbutton PRESS");
            state1.borrow_mut().to_audience_pushbutton_is_pressed = true;
            send_message(state1.borrow().to_message());
        }),
        Box::new(move || {
            log::info!("jimmy_pushbutton RELEASE");
            state2.borrow_mut().to_audience_pushbutton_is_pressed = false;
            send_message(state2.borrow().to_message());
        }),
    );

    let (state1, state2) = (Rc::clone(&state), Rc::clone(&state));
    let jimmy_pedal = Button::new(
        peripherals.pins.gpio19,
        Box::new(move || {
            log::info!("jimmy_pedal PRESS");
            state1.borrow_mut().to_band_pedal_is_pressed = true;
            send_message(state1.borrow().to_message());
        }),
        Box::new(move || {
            log::info!("jimmy_pedal RELEASE");
            state2.borrow_mut().to_band_pedal_is_pressed = false;
            send_message(state2.borrow().to_message());
        }),
    );

    let mike_state = Rc::new(RefCell::new(SimpleMicrophoneSenderState::default()));

    let (state1, state2) = (Rc::clone(&mike_state), Rc::clone(&mike_state));
    let mike_pushbutton = Button::new(
        peripherals.pins.gpio18,
        Box::new(move || {
            log::info!("mike_pushbutton PRESS");
            state1.borrow_mut().to_audience_pushbutton_is_pressed = true;
            send_message(state1.borrow().to_message());
        }),
        Box::new(move || {
            log::info!("mike_pushbutton RELEASE");
            state2.borrow_mut().to_audience_pushbutton_is_pressed = false;
            send_message(state2.borrow().to_message());
        }),
    );

    // 3. Initialize the Single-Threaded Async Executor
    let executor: LocalExecutor = LocalExecutor::default();

    log::info!("\nSENDER JIMMY version 1.0\n");

    // 4. Wrap your button tasks inside the executor block
    edge_executor::block_on(executor.run(async {
        // Run all button async loops concurrently
        let _ = futures::join!(
            Box::pin(monitor_button(jimmy_latch)),
            Box::pin(monitor_button(jimmy_pushbutton)),
            Box::pin(monitor_button(jimmy_pedal)),
            Box::pin(monitor_button(mike_pushbutton)),
        );
    }));
}

/// An async worker function that monitors a single GPIO pin for state changes.
async fn monitor_button<'a>(mut button: Button<'a>) {
    loop {
        button.wait_and_update().await;
    }
}

fn send_message(message: Message) {
    log::info!("send_message: {:?}", message);

    let result = unsafe {
        esp_now_send(
            RECEIVER_MAC.as_ptr(),
            &raw const message as *const u8,
            MESSAGE_SIZE,
        )
    };

    if result == ESP_OK {
        log::info!("Successfully sent {} bytes of message", MESSAGE_SIZE);
    } else {
        log::error!("Error sending message: {:?}", result);
    }
}
