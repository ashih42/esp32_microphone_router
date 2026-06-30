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

#[cfg(test)]
mod tests {
    use super::*;

    /// Go over all 8 permutations of the 3 boolean states.
    /// Requirements: https://docs.google.com/spreadsheets/d/1QiK6jzAJQySYgz_KvizED40fUrpU3yX_CyY_O5MObKY/edit?gid=0#gid=0
    #[test]
    fn test_routable_microphone_sender_state_to_message() {
        {
            let state = RoutableMicrophoneSenderState {
                to_audience_latch_is_pressed: false,
                to_band_pedal_is_pressed: false,
                to_audience_pushbutton_is_pressed: false,
            };

            assert!(matches!(
                state.to_message(),
                EspNowMessage::UpdateRoutableMicrophone { active: false, .. }
            ));
        }

        {
            let state = RoutableMicrophoneSenderState {
                to_audience_latch_is_pressed: false,
                to_band_pedal_is_pressed: false,
                to_audience_pushbutton_is_pressed: true,
            };

            assert!(matches!(
                state.to_message(),
                EspNowMessage::UpdateRoutableMicrophone {
                    active: true,
                    route: MicrophoneRoute::ToAudience,
                    ..
                }
            ));
        }

        {
            let state = RoutableMicrophoneSenderState {
                to_audience_latch_is_pressed: false,
                to_band_pedal_is_pressed: true,
                to_audience_pushbutton_is_pressed: false,
            };

            assert!(matches!(
                state.to_message(),
                EspNowMessage::UpdateRoutableMicrophone {
                    active: true,
                    route: MicrophoneRoute::ToBand,
                    ..
                }
            ));
        }

        {
            let state = RoutableMicrophoneSenderState {
                to_audience_latch_is_pressed: false,
                to_band_pedal_is_pressed: true,
                to_audience_pushbutton_is_pressed: true,
            };

            assert!(matches!(
                state.to_message(),
                EspNowMessage::UpdateRoutableMicrophone {
                    active: true,
                    route: MicrophoneRoute::ToBand,
                    ..
                }
            ));
        }

        {
            let state = RoutableMicrophoneSenderState {
                to_audience_latch_is_pressed: true,
                to_band_pedal_is_pressed: false,
                to_audience_pushbutton_is_pressed: false,
            };

            assert!(matches!(
                state.to_message(),
                EspNowMessage::UpdateRoutableMicrophone {
                    active: true,
                    route: MicrophoneRoute::ToAudience,
                    ..
                }
            ));
        }

        {
            let state = RoutableMicrophoneSenderState {
                to_audience_latch_is_pressed: true,
                to_band_pedal_is_pressed: false,
                to_audience_pushbutton_is_pressed: true,
            };

            assert!(matches!(
                state.to_message(),
                EspNowMessage::UpdateRoutableMicrophone {
                    active: true,
                    route: MicrophoneRoute::ToAudience,
                    ..
                }
            ));
        }

        {
            let state = RoutableMicrophoneSenderState {
                to_audience_latch_is_pressed: true,
                to_band_pedal_is_pressed: true,
                to_audience_pushbutton_is_pressed: false,
            };

            assert!(matches!(
                state.to_message(),
                EspNowMessage::UpdateRoutableMicrophone {
                    active: true,
                    route: MicrophoneRoute::ToBand,
                    ..
                }
            ));
        }

        {
            let state = RoutableMicrophoneSenderState {
                to_audience_latch_is_pressed: true,
                to_band_pedal_is_pressed: true,
                to_audience_pushbutton_is_pressed: true,
            };

            assert!(matches!(
                state.to_message(),
                EspNowMessage::UpdateRoutableMicrophone {
                    active: true,
                    route: MicrophoneRoute::ToBand,
                    ..
                }
            ));
        }
    }

    /// Go over all 2 permutations of the 1 boolean state.
    #[test]
    fn test_simple_microphone_sender_state_to_message() {
        {
            let state = SimpleMicrophoneSenderState {
                to_audience_pushbutton_is_pressed: false,
            };

            assert!(matches!(
                state.to_message(),
                EspNowMessage::UpdateSimpleMicrophone { active: false, .. }
            ));
        }

        {
            let state = SimpleMicrophoneSenderState {
                to_audience_pushbutton_is_pressed: true,
            };

            assert!(matches!(
                state.to_message(),
                EspNowMessage::UpdateSimpleMicrophone { active: true, .. }
            ));
        }
    }
}
