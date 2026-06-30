use std::fmt;

use crate::models::MicrophoneRoute;

/// There are 3 types of messages:
///   - reset_microphone: Reset the receiver's `last_message_id` field for a specific microphone.
///   - update_routable_microphone: Update the state of the routable microphone (with `message_id`).
///   - update_simple_microphone: Update the state of the simple microphone (with `message_id`).
///
/// Bad news: I cannot model `EspNowMessage` as a Rust enum and serialize it with `serde` and `bincode`
/// because `bincode` breaks the old nightly version of Rust Analyzer I need to use in this project with ESP32 Xtensa.
///
/// So, I will model `EspNowMessage` with `payload` as C union instead.
#[repr(C)]
pub struct EspNowMessage {
    pub payload: EspNowMessagePayload,
    pub header: EspNowMessageHeader,
}

pub const ESP_NOW_MESSAGE_SIZE: usize = std::mem::size_of::<EspNowMessage>();

impl fmt::Debug for EspNowMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("EspNowMessage");

        d.field("header", &self.header);

        match self.header {
            EspNowMessageHeader::ResetMicrophone => {
                d.field("payload", unsafe { &self.payload.reset_microphone });
            }
            EspNowMessageHeader::UpdateRoutableMicrophone => {
                d.field("payload", unsafe {
                    &self.payload.update_routable_microphone
                });
            }
            EspNowMessageHeader::UpdateSimpleMicrophone => {
                d.field("payload", unsafe { &self.payload.update_simple_microphone });
            }
        };

        d.finish()
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum EspNowMessageHeader {
    ResetMicrophone,
    UpdateRoutableMicrophone,
    UpdateSimpleMicrophone,
}

#[repr(C)]
pub union EspNowMessagePayload {
    pub reset_microphone: ResetMicrophonePayload,
    pub update_routable_microphone: UpdateRoutableMicrophonePayload,
    pub update_simple_microphone: UpdateSimpleMicrophonePayload,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ResetMicrophonePayload {
    pub microphone_id: MicrophoneId,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UpdateRoutableMicrophonePayload {
    pub message_id: u16,
    pub route: MicrophoneRoute,
    pub active: bool,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UpdateSimpleMicrophonePayload {
    pub message_id: u16,
    pub active: bool,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum MicrophoneId {
    RoutableMicrophone,
    SimpleMicrophone,
}

pub trait ToMessage {
    fn to_message(&self) -> EspNowMessage;
}
