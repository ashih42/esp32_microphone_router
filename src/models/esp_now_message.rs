use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU16, Ordering};

use crate::models::{MicrophoneType, RoutableMicrophoneLogicalState, SimpleMicrophoneLogicalState};

/// There are 3 types of messages:
///   - ResetMicrophone: Reset the receiver's `last_message_id` field for a specific microphone.
///   - UpdateRoutableMicrophone: Update the state of the routable microphone.
///   - UpdateSimpleMicrophone: Update the state of the simple microphone.
#[derive(Debug, Deserialize, Serialize)]
pub enum EspNowMessage {
    ResetMicrophone {
        microphone_type: MicrophoneType,
    },
    UpdateRoutableMicrophone {
        message_id: u16,
        state: RoutableMicrophoneLogicalState,
    },
    UpdateSimpleMicrophone {
        message_id: u16,
        state: SimpleMicrophoneLogicalState,
    },
}

pub trait ToMessage {
    fn to_message(&self) -> EspNowMessage;
}

impl EspNowMessage {
    /// Generate an auto-incremented u16 value to uniquely identify each message.
    /// This is useful for the receiver to determine if messages were received in the wrong order.
    pub fn generate_message_id() -> u16 {
        static COUNTER: AtomicU16 = AtomicU16::new(1);

        // Return the current value in `COUNTER`, and then increment `COUNTER`.
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
}
