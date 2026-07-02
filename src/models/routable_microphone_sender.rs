use esp_idf_hal::gpio::{Output, OutputPin, PinDriver};

use crate::{
    esp_now,
    models::{EspNowMessage, MicrophoneRoute, ToMessage},
};

pub struct RoutableMicrophoneSender<'a> {
    pub physical_state: RoutableMicrophoneSenderPhysicalState,
    hardware: RoutableMicrophoneSenderHardware<'a>,
}

impl<'a> RoutableMicrophoneSender<'a> {
    pub fn new<T, U>(to_audience_led_pin: T, to_band_led_pin: U) -> Self
    where
        T: OutputPin + 'a,
        U: OutputPin + 'a,
    {
        Self {
            physical_state: RoutableMicrophoneSenderPhysicalState::default(),
            hardware: RoutableMicrophoneSenderHardware::new(to_audience_led_pin, to_band_led_pin),
        }
    }

    pub fn update(&mut self) {
        // 1. Generate logical state.
        let logical_state = self.physical_state.to_logical_state();

        // 2. Update hardware.
        self.hardware.update(&logical_state);

        // 3. Send message over ESP-NOW.
        esp_now::send_message(logical_state.to_message());
    }
}

#[derive(Default, Debug)]
pub struct RoutableMicrophoneSenderPhysicalState {
    pub to_audience_latch_is_pressed: bool,
    pub to_audience_pushbutton_is_pressed: bool,
    pub to_band_pedal_is_pressed: bool,
}

impl RoutableMicrophoneSenderPhysicalState {
    pub fn flip_to_audience_latch(&mut self) {
        self.to_audience_latch_is_pressed = !self.to_audience_latch_is_pressed;
    }
}

pub struct RoutableMicrophoneSenderHardware<'a> {
    to_audience_led: PinDriver<'a, Output>,
    to_band_led: PinDriver<'a, Output>,
}

impl<'a> RoutableMicrophoneSenderHardware<'a> {
    fn new<T, U>(to_audience_led_pin: T, to_band_led_pin: U) -> Self
    where
        T: OutputPin + 'a,
        U: OutputPin + 'a,
    {
        let mut to_audience_led = PinDriver::output(to_audience_led_pin).unwrap();
        let mut to_band_led = PinDriver::output(to_band_led_pin).unwrap();

        to_audience_led.set_low().unwrap();
        to_band_led.set_low().unwrap();

        Self {
            to_audience_led,
            to_band_led,
        }
    }

    fn update(&mut self, logical_state: &RoutableMicrophoneLogicalState) {
        use RoutableMicrophoneLogicalState::{ActiveToAudience, ActiveToBand, Off};

        match logical_state {
            Off => {
                self.to_audience_led.set_low().unwrap();
                self.to_band_led.set_low().unwrap();
            }
            ActiveToAudience => {
                self.to_audience_led.set_high().unwrap();
                self.to_band_led.set_low().unwrap();
            }
            ActiveToBand => {
                self.to_audience_led.set_low().unwrap();
                self.to_band_led.set_high().unwrap();
            }
        }
    }
}

impl RoutableMicrophoneSenderPhysicalState {
    /// Reference: https://docs.google.com/spreadsheets/d/1QiK6jzAJQySYgz_KvizED40fUrpU3yX_CyY_O5MObKY/edit?gid=0#gid=0
    pub fn to_logical_state(&self) -> RoutableMicrophoneLogicalState {
        use RoutableMicrophoneLogicalState::{ActiveToAudience, ActiveToBand, Off};

        match (
            self.to_audience_latch_is_pressed,
            self.to_band_pedal_is_pressed,
            self.to_audience_pushbutton_is_pressed,
        ) {
            (false, false, false) => Off,
            (false, false, true) => ActiveToAudience,
            (false, true, false) => ActiveToBand,
            (false, true, true) => ActiveToBand,
            (true, false, false) => ActiveToAudience,
            (true, false, true) => ActiveToAudience,
            (true, true, false) => ActiveToBand,
            (true, true, true) => ActiveToBand,
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub enum RoutableMicrophoneLogicalState {
    #[default]
    Off,
    ActiveToAudience,
    ActiveToBand,
}

impl ToMessage for RoutableMicrophoneLogicalState {
    fn to_message(&self) -> EspNowMessage {
        let message_id = EspNowMessage::generate_message_id();

        match self {
            Self::Off => EspNowMessage::UpdateRoutableMicrophone {
                active: false,
                route: MicrophoneRoute::default(),
                message_id,
            },
            Self::ActiveToAudience => EspNowMessage::UpdateRoutableMicrophone {
                active: true,
                route: MicrophoneRoute::ToAudience,
                message_id,
            },
            Self::ActiveToBand => EspNowMessage::UpdateRoutableMicrophone {
                active: true,
                route: MicrophoneRoute::ToBand,
                message_id,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check all 8 possible physical states.
    /// Reference: https://docs.google.com/spreadsheets/d/1QiK6jzAJQySYgz_KvizED40fUrpU3yX_CyY_O5MObKY/edit?gid=0#gid=0
    #[test]
    fn test_physical_state_to_logical_state() {
        use RoutableMicrophoneLogicalState::{ActiveToAudience, ActiveToBand, Off};

        {
            let state = RoutableMicrophoneSenderPhysicalState {
                to_audience_latch_is_pressed: false,
                to_band_pedal_is_pressed: false,
                to_audience_pushbutton_is_pressed: false,
            };

            assert_eq!(state.to_logical_state(), Off);
        }

        {
            let state = RoutableMicrophoneSenderPhysicalState {
                to_audience_latch_is_pressed: false,
                to_band_pedal_is_pressed: false,
                to_audience_pushbutton_is_pressed: true,
            };

            assert_eq!(state.to_logical_state(), ActiveToAudience);
        }

        {
            let state = RoutableMicrophoneSenderPhysicalState {
                to_audience_latch_is_pressed: false,
                to_band_pedal_is_pressed: true,
                to_audience_pushbutton_is_pressed: false,
            };

            assert_eq!(state.to_logical_state(), ActiveToBand);
        }

        {
            let state = RoutableMicrophoneSenderPhysicalState {
                to_audience_latch_is_pressed: false,
                to_band_pedal_is_pressed: true,
                to_audience_pushbutton_is_pressed: true,
            };

            assert_eq!(state.to_logical_state(), ActiveToBand);
        }

        {
            let state = RoutableMicrophoneSenderPhysicalState {
                to_audience_latch_is_pressed: true,
                to_band_pedal_is_pressed: false,
                to_audience_pushbutton_is_pressed: false,
            };

            assert_eq!(state.to_logical_state(), ActiveToAudience);
        }

        {
            let state = RoutableMicrophoneSenderPhysicalState {
                to_audience_latch_is_pressed: true,
                to_band_pedal_is_pressed: false,
                to_audience_pushbutton_is_pressed: true,
            };

            assert_eq!(state.to_logical_state(), ActiveToAudience);
        }

        {
            let state = RoutableMicrophoneSenderPhysicalState {
                to_audience_latch_is_pressed: true,
                to_band_pedal_is_pressed: true,
                to_audience_pushbutton_is_pressed: false,
            };

            assert_eq!(state.to_logical_state(), ActiveToBand);
        }

        {
            let state = RoutableMicrophoneSenderPhysicalState {
                to_audience_latch_is_pressed: true,
                to_band_pedal_is_pressed: true,
                to_audience_pushbutton_is_pressed: true,
            };

            assert_eq!(state.to_logical_state(), ActiveToBand);
        }
    }

    /// Check all 3 possible logical states.
    #[test]
    fn test_logical_state_to_message() {
        use RoutableMicrophoneLogicalState::{ActiveToAudience, ActiveToBand, Off};

        assert!(matches!(
            Off.to_message(),
            EspNowMessage::UpdateRoutableMicrophone { active: false, .. }
        ));

        assert!(matches!(
            ActiveToAudience.to_message(),
            EspNowMessage::UpdateRoutableMicrophone {
                active: true,
                route: MicrophoneRoute::ToAudience,
                ..
            }
        ));

        assert!(matches!(
            ActiveToBand.to_message(),
            EspNowMessage::UpdateRoutableMicrophone {
                active: true,
                route: MicrophoneRoute::ToBand,
                ..
            }
        ));
    }
}
