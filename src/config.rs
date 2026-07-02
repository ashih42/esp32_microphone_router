pub const SENDER_JIMMY_MAC: [u8; 6] = [0xF8, 0xB3, 0xB7, 0x44, 0x4D, 0x08];
pub const SENDER_MIKE_MAC: [u8; 6] = [0x30, 0x76, 0xF5, 0xE7, 0x7B, 0x04];
pub const RECEIVER_MAC: [u8; 6] = [0xF8, 0xB3, 0xB7, 0x44, 0xB9, 0xE4];

pub const WIFI_CHANNEL: u8 = 1;

pub const ESP_NOW_PMK: [u8; 16] = validate_secret_as_16_bytes(env!("ESP_NOW_PMK"));
pub const ESP_NOW_LMK: [u8; 16] = validate_secret_as_16_bytes(env!("ESP_NOW_LMK"));

/// This function is used to validate the `PMK` (Primary Master Key) and `LMK` (Secret Master Key)
/// secret values loaded from environment. ESP-NOW requires both must be exactly 16 bytes.
const fn validate_secret_as_16_bytes(secret: &str) -> [u8; 16] {
    let bytes = secret.as_bytes();

    if bytes.len() != 16 {
        panic!("Secret key must be exactly 16 bytes!");
    }

    let array_with_16_bytes = bytes.as_ptr() as *const [u8; 16];

    unsafe { *array_with_16_bytes }
}
