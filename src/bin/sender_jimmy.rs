use edge_executor::LocalExecutor;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use std::{cell::RefCell, rc::Rc};

use esp32_microphone_router::{
    button::Button,
    models::{MicrophoneSender, RoutableMicrophoneSender},
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

    // --------------------------------------------------------------------------------------------

    // Create the top-level sender object.
    let sender = Rc::new(RefCell::new(RoutableMicrophoneSender::new(
        peripherals.pins.gpio17,
        peripherals.pins.gpio16,
    )));

    sender.borrow_mut().initialize();

    // Set up callbacks on press/release of the button serving as the latch to audience.
    let sender1 = Rc::clone(&sender);
    let mut to_audience_latch = Button::new(
        peripherals.pins.gpio22,
        Some(Box::new(move || {
            log::info!("to_audience_latch PRESS");

            let mut sender = sender1.borrow_mut();

            sender.physical_state.flip_to_audience_latch();
            sender.update();
        })),
        None,
    );

    // Set up callbacks on press/release of the button serving as the button to audience.
    let (sender1, sender2) = (Rc::clone(&sender), Rc::clone(&sender));
    let mut to_audience_pushbutton = Button::new(
        peripherals.pins.gpio21,
        Some(Box::new(move || {
            log::info!("to_audience_pushbutton PRESS");

            let mut sender = sender1.borrow_mut();

            sender.physical_state.to_audience_pushbutton_is_pressed = true;
            sender.update();
        })),
        Some(Box::new(move || {
            log::info!("to_audience_pushbutton RELEASE");

            let mut sender = sender2.borrow_mut();

            sender.physical_state.to_audience_pushbutton_is_pressed = false;
            sender.update();
        })),
    );

    // Set up callbacks on press/release of the button serving as the pedal to band.
    let (sender1, sender2) = (Rc::clone(&sender), Rc::clone(&sender));
    let mut to_band_pedal = Button::new(
        peripherals.pins.gpio19,
        Some(Box::new(move || {
            log::info!("to_band_pedal PRESS");

            let mut sender = sender1.borrow_mut();

            sender.physical_state.to_band_pedal_is_pressed = true;
            sender.update();
        })),
        Some(Box::new(move || {
            log::info!("to_band_pedal RELEASE");

            let mut sender = sender2.borrow_mut();

            sender.physical_state.to_band_pedal_is_pressed = false;
            sender.update();
        })),
    );

    log::info!("SENDER JIMMY BEGIN");

    // 3. Initialize a single-threaded async executor.
    let executor: LocalExecutor = LocalExecutor::default();

    // Run all button async loops concurrently forever.
    edge_executor::block_on(executor.run(async {
        let _ = futures::join!(
            Box::pin(to_audience_latch.run()),
            Box::pin(to_audience_pushbutton.run()),
            Box::pin(to_band_pedal.run()),
        );
    }));
}
