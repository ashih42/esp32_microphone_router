use serde::{Deserialize, Serialize};

use crate::models::MicrophoneRoute;

/// There are 3 types of messages:
///   - ResetMicrophone: Reset the receiver's `last_message_id` field for a specific microphone.
///   - UpdateRoutableMicrophone: Update the state of the routable microphone (with `message_id`).
///   - UpdateSimpleMicrophone: Update the state of the simple microphone (with `message_id`).
///
/// Note: I cannot serialize `EspNowMessage` to bytes using `serde` and `bincode`
/// because `bincode` breaks the old nightly version of Rust Analyzer I need to use in this project with ESP32 Xtensa.
/// Fortunately, I can use `postcard` instead of `bincode`.
#[derive(Debug, Deserialize, Serialize)]
pub enum EspNowMessage {
    ResetMicrophone {
        microphone_type: MicrophoneType,
    },
    UpdateRoutableMicrophone {
        message_id: u16,
        route: MicrophoneRoute,
        active: bool,
    },
    UpdateSimpleMicrophone {
        message_id: u16,
        active: bool,
    },
}

#[repr(u8)]
#[derive(Debug, Deserialize, Serialize)]
pub enum MicrophoneType {
    RoutableMicrophone,
    SimpleMicrophone,
}

pub trait ToMessage {
    fn to_message(&self) -> EspNowMessage;
}
