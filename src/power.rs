use core::ffi::c_void;
use esp_idf_sys::{ESP_OK, esp_pm_config_t, esp_pm_configure};

const POWER_MANAGEMENT_CONFIG: esp_pm_config_t = esp_pm_config_t {
    max_freq_mhz: 80,
    min_freq_mhz: 80,
    light_sleep_enable: false,
};

/// Set the CPU frequency to the lowest baseline value (80 MHz) for wifi to work,
/// and limit the allowed frequency range so it never fluctuates.
/// This may help reduce heating buildup around the physical device.
pub fn limit_cpu_speed() {
    let config = &POWER_MANAGEMENT_CONFIG as *const esp_pm_config_t as *const c_void;

    if unsafe { esp_pm_configure(config) } != ESP_OK {
        panic!("esp_pm_configure() failed.")
    }
}
