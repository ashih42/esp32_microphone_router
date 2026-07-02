use serde::{Deserialize, Serialize};

use crate::models::{EspNowMessage, ToMessage};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum RoutableMicrophoneLogicalState {
    #[default]
    Muted,
    ActiveToAudience,
    ActiveToBand,
}

impl ToMessage for RoutableMicrophoneLogicalState {
    fn to_message(&self) -> EspNowMessage {
        EspNowMessage::UpdateRoutableMicrophone {
            state: *self,
            message_id: EspNowMessage::generate_message_id(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum SimpleMicrophoneLogicalState {
    #[default]
    Muted,
    Active,
}

impl ToMessage for SimpleMicrophoneLogicalState {
    fn to_message(&self) -> EspNowMessage {
        EspNowMessage::UpdateSimpleMicrophone {
            state: *self,
            message_id: EspNowMessage::generate_message_id(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Deserialize, Serialize)]
pub enum MicrophoneType {
    RoutableMicrophone,
    SimpleMicrophone,
}
