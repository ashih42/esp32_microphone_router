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
        EspNowMessage, MicrophoneType, RoutableMicrophoneSender,
        RoutableMicrophoneSenderPhysicalState, ToMessage,
    },
    power,
};

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

    esp_now::initialize_esp_now_as_sender();

    // --------------------------------------------------------------------------------------------

    // Create the top-level master object.
    let sender = Rc::new(RefCell::new(RoutableMicrophoneSender::new(
        peripherals.pins.gpio17,
        peripherals.pins.gpio16,
    )));

    // Set up callbacks on press/release of the button serving as the latch to audience.
    let sender1 = Rc::clone(&sender);
    let mut jimmy_latch = Button::new(
        peripherals.pins.gpio22,
        Some(Box::new(move || {
            log::info!("jimmy_latch PRESS");

            let mut sender = sender1.borrow_mut();

            sender.physical_state.flip_to_audience_latch();
            sender.update();
        })),
        None,
    );

    // Set up callbacks on press/release of the button serving as the button to audience.
    let (sender1, sender2) = (Rc::clone(&sender), Rc::clone(&sender));
    let mut jimmy_pushbutton = Button::new(
        peripherals.pins.gpio21,
        Some(Box::new(move || {
            log::info!("jimmy_pushbutton PRESS");

            let mut sender = sender1.borrow_mut();

            sender.physical_state.to_audience_pushbutton_is_pressed = true;
            sender.update();
        })),
        Some(Box::new(move || {
            log::info!("jimmy_pushbutton RELEASE");

            let mut sender = sender2.borrow_mut();

            sender.physical_state.to_audience_pushbutton_is_pressed = false;
            sender.update();
        })),
    );

    // Set up callbacks on press/release of the button serving as the pedal to band.
    let (sender1, sender2) = (Rc::clone(&sender), Rc::clone(&sender));
    let mut jimmy_pedal = Button::new(
        peripherals.pins.gpio19,
        Some(Box::new(move || {
            log::info!("jimmy_pedal PRESS");

            let mut sender = sender1.borrow_mut();

            sender.physical_state.to_band_pedal_is_pressed = true;
            sender.update();
        })),
        Some(Box::new(move || {
            log::info!("jimmy_pedal RELEASE");

            let mut sender = sender2.borrow_mut();

            sender.physical_state.to_band_pedal_is_pressed = false;
            sender.update();
        })),
    );

    // TODO: Move Mike operations to a separate binary.
    // let mike_state = Rc::new(RefCell::new(SimpleMicrophoneSenderState::default()));

    // let (state1, state2) = (Rc::clone(&mike_state), Rc::clone(&mike_state));
    // let mike_pushbutton = Button::new(
    //     peripherals.pins.gpio18,
    //     Box::new(move || {
    //         log::info!("mike_pushbutton PRESS");
    //         state1.borrow_mut().to_audience_pushbutton_is_pressed = true;
    //         send_message(state1.borrow().to_message());
    //     }),
    //     Box::new(move || {
    //         log::info!("mike_pushbutton RELEASE");
    //         state2.borrow_mut().to_audience_pushbutton_is_pressed = false;
    //         send_message(state2.borrow().to_message());
    //     }),
    // );

    log::info!("SENDER JIMMY BEGIN");

    // Send a message to reset the microphone for Jimmy.
    esp_now::send_message(EspNowMessage::ResetMicrophone {
        microphone_type: MicrophoneType::RoutableMicrophone,
    });

    // Send a message to reset the microphone for Mike.
    // send_message(EspNowMessage::ResetMicrophone {
    //     microphone_type: MicrophoneType::SimpleMicrophone,
    // });

    // 3. Initialize a single-threaded async executor.
    let executor: LocalExecutor = LocalExecutor::default();

    // Run all button async loops concurrently forever.
    edge_executor::block_on(executor.run(async {
        let _ = futures::join!(
            Box::pin(jimmy_latch.run()),
            Box::pin(jimmy_pushbutton.run()),
            Box::pin(jimmy_pedal.run()),
            // Box::pin(monitor_button(mike_pushbutton)),
        );
    }));
}

// async fn monitor_button<'a>(mut button: Button<'a>) {
//     loop {
//         button.wait_and_update().await;
//     }
// }

// /// Send a message over ESP-NOW.
// /// Note: I don't bother setting up a callback function to check if receiver sent back an ACK packet.
// fn send_message(message: EspNowMessage) {
//     log::info!("send_message: {:?}", message);

//     // ESP-NOW max payload size is 250 bytes.
//     let mut buffer = [0_u8; 250];

//     match postcard::to_slice(&message, &mut buffer) {
//         Err(err) => {
//             log::error!("postcard::to_slice failed: {}", err);
//         }
//         Ok(data) => {
//             let status = unsafe { esp_now_send(RECEIVER_MAC.as_ptr(), data.as_ptr(), data.len()) };

//             if status == ESP_OK {
//                 log::info!("Successfully sent {} bytes of message", data.len());
//             } else {
//                 log::error!("esp_now_send() failed: {:?}", status);
//             }
//         }
//     }
// }
