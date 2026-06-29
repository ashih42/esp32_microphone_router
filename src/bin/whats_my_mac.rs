use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{EspWifi, WifiDeviceId},
};
use std::{thread, time::Duration};

fn main() -> ! {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let sys_loop = EspSystemEventLoop::take().expect("Failed to take EspSystemEventLoop.");
    let nvs = EspDefaultNvsPartition::take().expect("Failed to take EspDefaultNvsPartition.");
    let peripherals = Peripherals::take().expect("Failed to take Peripherals.");

    let esp_wifi =
        EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).expect("Failed to create EspWifi.");

    println!();

    // Print the MAC address (6 bytes) as hexadecimals.
    match esp_wifi.get_mac(WifiDeviceId::Sta) {
        Ok(mac) => {
            println!(
                "Your ESP32 MAC address is: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
            );
        }
        Err(err) => {
            println!("Failed to get MAC address: {:?}", err);
        }
    };

    loop {
        thread::sleep(Duration::from_secs(10));
    }
}
