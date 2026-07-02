use esp_idf_hal::gpio::{Output, OutputPin, PinDriver};

use crate::{
    esp_now,
    models::{EspNowMessage, MicrophoneSender, MicrophoneType, SimpleMicrophoneState, ToMessage},
};

pub struct SimpleMicrophoneSender<'a> {
    pub input: SimpleMicrophoneSenderInput,
    hardware: SimpleMicrophoneSenderHardware<'a>,
}

impl<'a> SimpleMicrophoneSender<'a> {
    pub fn new<T>(active_led_pin: T) -> Self
    where
        T: OutputPin + 'a,
    {
        Self {
            input: SimpleMicrophoneSenderInput::default(),
            hardware: SimpleMicrophoneSenderHardware::new(active_led_pin),
        }
    }
}

impl<'a> MicrophoneSender for SimpleMicrophoneSender<'a> {
    fn initialize(&mut self) {
        // Turn off LEDs.
        self.hardware.initialize();

        // Set up ESP-NOW.
        esp_now::initialize_esp_now_as_sender();

        // Send a message telling the receiver to reset this microphone.
        esp_now::send_message(EspNowMessage::ResetMicrophone {
            microphone_type: MicrophoneType::SimpleMicrophone,
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
pub struct SimpleMicrophoneSenderInput {
    pub active_pushbutton_is_pressed: bool,
}

struct SimpleMicrophoneSenderHardware<'a> {
    active_led: PinDriver<'a, Output>,
}

impl<'a> SimpleMicrophoneSenderHardware<'a> {
    fn new<T>(active_led_pin: T) -> Self
    where
        T: OutputPin + 'a,
    {
        Self {
            active_led: PinDriver::output(active_led_pin).unwrap(),
        }
    }

    /// Turn off LED.
    fn initialize(&mut self) {
        self.active_led.set_low().unwrap();
    }

    /// Update LED.
    fn flush(&mut self, state: &SimpleMicrophoneState) {
        use SimpleMicrophoneState::{Active, Muted};

        match state {
            Muted => {
                self.active_led.set_low().unwrap();
            }
            Active => {
                self.active_led.set_high().unwrap();
            }
        }
    }
}

impl SimpleMicrophoneSenderInput {
    /// Reference: https://docs.google.com/spreadsheets/d/1QiK6jzAJQySYgz_KvizED40fUrpU3yX_CyY_O5MObKY/edit?gid=0#gid=0
    pub fn to_state(&self) -> SimpleMicrophoneState {
        use SimpleMicrophoneState::{Active, Muted};

        match self.active_pushbutton_is_pressed {
            false => Muted,
            true => Active,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check all 2 possible input values.
    /// Reference: https://docs.google.com/spreadsheets/d/1QiK6jzAJQySYgz_KvizED40fUrpU3yX_CyY_O5MObKY/edit?gid=0#gid=0
    #[test]
    fn test_input_to_state() {
        use SimpleMicrophoneState::{Active, Muted};

        {
            let input = SimpleMicrophoneSenderInput {
                active_pushbutton_is_pressed: false,
            };

            assert_eq!(input.to_state(), Muted);
        }

        {
            let input = SimpleMicrophoneSenderInput {
                active_pushbutton_is_pressed: true,
            };

            assert_eq!(input.to_state(), Active);
        }
    }
}
