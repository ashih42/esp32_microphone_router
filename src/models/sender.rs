use std::sync::atomic::{AtomicU16, Ordering};

use crate::models::{
    esp_now_message::{EspNowMessage, ToMessage},
    microphone::MicrophoneRoute,
};

#[derive(Default, Debug)]
pub struct RoutableMicrophoneSenderState {
    pub to_audience_latch_is_pressed: bool,
    pub to_audience_pushbutton_is_pressed: bool,
    pub to_band_pedal_is_pressed: bool,
}

impl ToMessage for RoutableMicrophoneSenderState {
    fn to_message(&self) -> EspNowMessage {
        let message_id = generate_message_id();

        match (
            self.to_audience_latch_is_pressed,
            self.to_band_pedal_is_pressed,
            self.to_audience_pushbutton_is_pressed,
        ) {
            (false, false, false) => EspNowMessage::UpdateRoutableMicrophone {
                active: false,
                route: MicrophoneRoute::default(),
                message_id,
            },
            (false, false, true) => EspNowMessage::UpdateRoutableMicrophone {
                active: true,
                route: MicrophoneRoute::ToAudience,
                message_id,
            },
            (false, true, false) => EspNowMessage::UpdateRoutableMicrophone {
                active: true,
                route: MicrophoneRoute::ToBand,
                message_id,
            },
            (false, true, true) => EspNowMessage::UpdateRoutableMicrophone {
                active: true,
                route: MicrophoneRoute::ToBand,
                message_id,
            },
            (true, false, false) => EspNowMessage::UpdateRoutableMicrophone {
                active: true,
                route: MicrophoneRoute::ToAudience,
                message_id,
            },
            (true, false, true) => EspNowMessage::UpdateRoutableMicrophone {
                active: true,
                route: MicrophoneRoute::ToAudience,
                message_id,
            },
            (true, true, false) => EspNowMessage::UpdateRoutableMicrophone {
                active: true,
                route: MicrophoneRoute::ToBand,
                message_id,
            },
            (true, true, true) => EspNowMessage::UpdateRoutableMicrophone {
                active: true,
                route: MicrophoneRoute::ToBand,
                message_id,
            },
        }
    }
}

#[derive(Default, Debug)]
pub struct SimpleMicrophoneSenderState {
    pub to_audience_pushbutton_is_pressed: bool,
}

impl ToMessage for SimpleMicrophoneSenderState {
    fn to_message(&self) -> EspNowMessage {
        EspNowMessage::UpdateSimpleMicrophone {
            active: self.to_audience_pushbutton_is_pressed,
            message_id: generate_message_id(),
        }
    }
}

fn generate_message_id() -> u16 {
    static COUNTER: AtomicU16 = AtomicU16::new(1);

    // Return the current value in `COUNTER`, and then increment `COUNTER`.
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
