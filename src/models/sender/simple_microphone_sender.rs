use esp_idf_hal::gpio::{Output, OutputPin, PinDriver};

use crate::{
    esp_now,
    models::{
        EspNowMessage, MicrophoneSender, MicrophoneType, SimpleMicrophoneLogicalState, ToMessage,
    },
};

pub struct SimpleMicrophoneSender<'a> {
    pub physical_state: SimpleMicrophoneSenderPhysicalState,
    hardware: SimpleMicrophoneSenderHardware<'a>,
}

impl<'a> SimpleMicrophoneSender<'a> {
    pub fn new<T>(active_led_pin: T) -> Self
    where
        T: OutputPin + 'a,
    {
        Self {
            physical_state: SimpleMicrophoneSenderPhysicalState::default(),
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

    /// Whenever after `physical_state` was updated by button press/release events, call this function to
    /// propagate the effects to other parts of the system.
    fn update(&mut self) {
        // 1. Generate logical state from physical state.
        let logical_state = self.physical_state.to_logical_state();

        // 2. Use logical state to update hardware.
        self.hardware.flush(&logical_state);

        // 3. Use logical state to create message to send over ESP-NOW.
        esp_now::send_message(logical_state.to_message());
    }
}

#[derive(Default, Debug)]
pub struct SimpleMicrophoneSenderPhysicalState {
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
    fn flush(&mut self, logical_state: &SimpleMicrophoneLogicalState) {
        use SimpleMicrophoneLogicalState::{Active, Muted};

        match logical_state {
            Muted => {
                self.active_led.set_low().unwrap();
            }
            Active => {
                self.active_led.set_high().unwrap();
            }
        }
    }
}

impl SimpleMicrophoneSenderPhysicalState {
    /// Reference: https://docs.google.com/spreadsheets/d/1QiK6jzAJQySYgz_KvizED40fUrpU3yX_CyY_O5MObKY/edit?gid=0#gid=0
    pub fn to_logical_state(&self) -> SimpleMicrophoneLogicalState {
        use SimpleMicrophoneLogicalState::{Active, Muted};

        match self.active_pushbutton_is_pressed {
            false => Muted,
            true => Active,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check all 2 possible physical states.
    /// Reference: https://docs.google.com/spreadsheets/d/1QiK6jzAJQySYgz_KvizED40fUrpU3yX_CyY_O5MObKY/edit?gid=0#gid=0
    #[test]
    fn test_physical_state_to_logical_state() {
        use SimpleMicrophoneLogicalState::{Active, Muted};

        {
            let state = SimpleMicrophoneSenderPhysicalState {
                active_pushbutton_is_pressed: false,
            };

            assert_eq!(state.to_logical_state(), Muted);
        }

        {
            let state = SimpleMicrophoneSenderPhysicalState {
                active_pushbutton_is_pressed: true,
            };

            assert_eq!(state.to_logical_state(), Active);
        }
    }
}
