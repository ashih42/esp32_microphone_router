use crate::models::{
    message::{Message, ToMessage},
    microphone::MicrophoneRoute,
};

#[derive(Default, Debug)]
pub struct RoutableMicrophoneSenderState {
    pub to_audience_latch_is_pressed: bool,
    pub to_audience_pushbutton_is_pressed: bool,
    pub to_band_pedal_is_pressed: bool,
}

impl ToMessage for RoutableMicrophoneSenderState {
    fn to_message(&self) -> Message {
        let message_id = Message::generate_message_id();

        match (
            self.to_audience_latch_is_pressed,
            self.to_band_pedal_is_pressed,
            self.to_audience_pushbutton_is_pressed,
        ) {
            (false, false, false) => Message {
                active: false,
                route: Some(MicrophoneRoute::default()),
                message_id,
            },
            (false, false, true) => Message {
                active: true,
                route: Some(MicrophoneRoute::ToAudience),
                message_id,
            },
            (false, true, false) => Message {
                active: true,
                route: Some(MicrophoneRoute::ToBand),
                message_id,
            },
            (false, true, true) => Message {
                active: true,
                route: Some(MicrophoneRoute::ToBand),
                message_id,
            },
            (true, false, false) => Message {
                active: true,
                route: Some(MicrophoneRoute::ToAudience),
                message_id,
            },
            (true, false, true) => Message {
                active: true,
                route: Some(MicrophoneRoute::ToAudience),
                message_id,
            },
            (true, true, false) => Message {
                active: true,
                route: Some(MicrophoneRoute::ToBand),
                message_id,
            },
            (true, true, true) => Message {
                active: true,
                route: Some(MicrophoneRoute::ToBand),
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
    fn to_message(&self) -> Message {
        Message {
            active: self.to_audience_pushbutton_is_pressed,
            route: None,
            message_id: Message::generate_message_id(),
        }
    }
}
