use esp_idf_sys::{
    ESP_OK, esp_now_add_peer, esp_now_init, esp_now_peer_info, esp_now_peer_info_t,
    esp_now_set_pmk, wifi_interface_t_WIFI_IF_STA,
};

use crate::config::{
    ESP_NOW_LMK, ESP_NOW_PMK, RECEIVER_MAC, SENDER_JIMMY_MAC, SENDER_MIKE_MAC, WIFI_CHANNEL,
};

pub fn initialize_esp_now_as_sender() {
    if unsafe { esp_now_init() } != ESP_OK {
        panic!("esp_now_it() failed.");
    }

    if unsafe { esp_now_set_pmk(ESP_NOW_PMK.as_ptr()) } != ESP_OK {
        panic!("Failed to set PMK.");
    }

    // Sender needs to register the receiver to send unicast messages.
    if unsafe { esp_now_add_peer(&RECEIVER_PEER_INFO.0) } != ESP_OK {
        panic!("esp_now_add_peer() failed.");
    }
}

pub fn initialize_esp_now_as_receiver() {
    if unsafe { esp_now_init() } != ESP_OK {
        panic!("esp_now_it() failed.");
    }

    if unsafe { esp_now_set_pmk(ESP_NOW_PMK.as_ptr()) } != ESP_OK {
        panic!("Failed to set PMK.");
    }

    // Receiver needs to register the senders to use encryption.
    if unsafe { esp_now_add_peer(&SENDER_JIMMY_PEER_INFO.0) } != ESP_OK {
        panic!("esp_now_add_peer() failed.");
    }
    if unsafe { esp_now_add_peer(&SENDER_MIKE_PEER_INFO.0) } != ESP_OK {
        panic!("esp_now_add_peer() failed.");
    }
}

/// This is a newtype to wrap `esp_now_peer_info` so I can implement Sync trait,
/// so I can instantiate it as static variable.
struct EspNowPeerInfo(esp_now_peer_info);

unsafe impl Sync for EspNowPeerInfo {}

static RECEIVER_PEER_INFO: EspNowPeerInfo = EspNowPeerInfo(esp_now_peer_info_t {
    peer_addr: RECEIVER_MAC,
    channel: WIFI_CHANNEL,
    ifidx: wifi_interface_t_WIFI_IF_STA,
    encrypt: true,
    lmk: ESP_NOW_LMK,
    priv_: std::ptr::null_mut(),
});

static SENDER_JIMMY_PEER_INFO: EspNowPeerInfo = EspNowPeerInfo(esp_now_peer_info_t {
    peer_addr: SENDER_JIMMY_MAC,
    channel: WIFI_CHANNEL,
    ifidx: wifi_interface_t_WIFI_IF_STA,
    encrypt: true,
    lmk: ESP_NOW_LMK,
    priv_: std::ptr::null_mut(),
});

static SENDER_MIKE_PEER_INFO: EspNowPeerInfo = EspNowPeerInfo(esp_now_peer_info_t {
    peer_addr: SENDER_MIKE_MAC,
    channel: WIFI_CHANNEL,
    ifidx: wifi_interface_t_WIFI_IF_STA,
    encrypt: true,
    lmk: ESP_NOW_LMK,
    priv_: std::ptr::null_mut(),
});
