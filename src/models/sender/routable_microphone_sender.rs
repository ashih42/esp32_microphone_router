use esp_idf_hal::gpio::{Output, OutputPin, PinDriver};

use crate::{
    esp_now,
    models::{EspNowMessage, MicrophoneSender, MicrophoneType, RoutableMicrophoneState, ToMessage},
};

pub struct RoutableMicrophoneSender<'a> {
    pub input: RoutableMicrophoneSenderInput,
    hardware: RoutableMicrophoneSenderHardware<'a>,
}

impl<'a> RoutableMicrophoneSender<'a> {
    pub fn new<T, U>(to_audience_led_pin: T, to_band_led_pin: U) -> Self
    where
        T: OutputPin + 'a,
        U: OutputPin + 'a,
    {
        Self {
            input: RoutableMicrophoneSenderInput::default(),
            hardware: RoutableMicrophoneSenderHardware::new(to_audience_led_pin, to_band_led_pin),
        }
    }
}

impl<'a> MicrophoneSender for RoutableMicrophoneSender<'a> {
    fn initialize(&mut self) {
        // Turn off LEDs.
        self.hardware.initialize();

        // Set up ESP-NOW.
        esp_now::initialize_esp_now_as_sender();

        // Send a message telling the receiver to reset this microphone.
        esp_now::send_message(EspNowMessage::ResetMicrophone {
            microphone_type: MicrophoneType::RoutableMicrophone,
        });
    }

    /// Whenever after `input` was updated by button press/release events, call this function to
    /// propagate the effects to other parts of the system.
    fn update(&mut self) {
        // 1. Generate state from input.
        let state = self.input.to_state();

        // 2. Use state to update hardware.
        self.hardware.flush(&state);

        // 3. Use state to create a message to send over ESP-NOW.
        esp_now::send_message(state.to_message());
    }
}

#[derive(Default, Debug)]
pub struct RoutableMicrophoneSenderInput {
    pub to_audience_latch_is_pressed: bool,
    pub to_audience_pushbutton_is_pressed: bool,
    pub to_band_pedal_is_pressed: bool,
}

impl RoutableMicrophoneSenderInput {
    pub fn flip_to_audience_latch(&mut self) {
        self.to_audience_latch_is_pressed = !self.to_audience_latch_is_pressed;
    }
}

struct RoutableMicrophoneSenderHardware<'a> {
    to_audience_led: PinDriver<'a, Output>,
    to_band_led: PinDriver<'a, Output>,
}

impl<'a> RoutableMicrophoneSenderHardware<'a> {
    fn new<T, U>(to_audience_led_pin: T, to_band_led_pin: U) -> Self
    where
        T: OutputPin + 'a,
        U: OutputPin + 'a,
    {
        Self {
            to_audience_led: PinDriver::output(to_audience_led_pin).unwrap(),
            to_band_led: PinDriver::output(to_band_led_pin).unwrap(),
        }
    }

    /// Turn off LEDs.
    fn initialize(&mut self) {
        self.to_audience_led.set_low().unwrap();
        self.to_band_led.set_low().unwrap();
    }

    /// Update LEDs.
    fn flush(&mut self, state: &RoutableMicrophoneState) {
        use RoutableMicrophoneState::{ActiveToAudience, ActiveToBand, Muted};

        match state {
            Muted => {
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

impl RoutableMicrophoneSenderInput {
    /// Reference: https://docs.google.com/spreadsheets/d/1QiK6jzAJQySYgz_KvizED40fUrpU3yX_CyY_O5MObKY/edit?gid=0#gid=0
    pub fn to_state(&self) -> RoutableMicrophoneState {
        use RoutableMicrophoneState::{ActiveToAudience, ActiveToBand, Muted};

        match (
            self.to_audience_latch_is_pressed,
            self.to_band_pedal_is_pressed,
            self.to_audience_pushbutton_is_pressed,
        ) {
            (false, false, false) => Muted,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Check all 8 possible input values.
    /// Reference: https://docs.google.com/spreadsheets/d/1QiK6jzAJQySYgz_KvizED40fUrpU3yX_CyY_O5MObKY/edit?gid=0#gid=0
    #[test]
    fn test_input_to_state() {
        use RoutableMicrophoneState::{ActiveToAudience, ActiveToBand, Muted};

        {
            let input = RoutableMicrophoneSenderInput {
                to_audience_latch_is_pressed: false,
                to_band_pedal_is_pressed: false,
                to_audience_pushbutton_is_pressed: false,
            };

            assert_eq!(input.to_state(), Muted);
        }

        {
            let input = RoutableMicrophoneSenderInput {
                to_audience_latch_is_pressed: false,
                to_band_pedal_is_pressed: false,
                to_audience_pushbutton_is_pressed: true,
            };

            assert_eq!(input.to_state(), ActiveToAudience);
        }

        {
            let input = RoutableMicrophoneSenderInput {
                to_audience_latch_is_pressed: false,
                to_band_pedal_is_pressed: true,
                to_audience_pushbutton_is_pressed: false,
            };

            assert_eq!(input.to_state(), ActiveToBand);
        }

        {
            let input = RoutableMicrophoneSenderInput {
                to_audience_latch_is_pressed: false,
                to_band_pedal_is_pressed: true,
                to_audience_pushbutton_is_pressed: true,
            };

            assert_eq!(input.to_state(), ActiveToBand);
        }

        {
            let input = RoutableMicrophoneSenderInput {
                to_audience_latch_is_pressed: true,
                to_band_pedal_is_pressed: false,
                to_audience_pushbutton_is_pressed: false,
            };

            assert_eq!(input.to_state(), ActiveToAudience);
        }

        {
            let input = RoutableMicrophoneSenderInput {
                to_audience_latch_is_pressed: true,
                to_band_pedal_is_pressed: false,
                to_audience_pushbutton_is_pressed: true,
            };

            assert_eq!(input.to_state(), ActiveToAudience);
        }

        {
            let input = RoutableMicrophoneSenderInput {
                to_audience_latch_is_pressed: true,
                to_band_pedal_is_pressed: true,
                to_audience_pushbutton_is_pressed: false,
            };

            assert_eq!(input.to_state(), ActiveToBand);
        }

        {
            let input = RoutableMicrophoneSenderInput {
                to_audience_latch_is_pressed: true,
                to_band_pedal_is_pressed: true,
                to_audience_pushbutton_is_pressed: true,
            };

            assert_eq!(input.to_state(), ActiveToBand);
        }
    }
}
