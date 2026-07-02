use esp_idf_hal::gpio::{Output, OutputPin, PinDriver};

use crate::{
    esp_now,
    models::{
        EspNowMessage, MicrophoneRoute, MicrophoneType,
        microphone::{RoutableMicrophone, SimpleMicrophone},
    },
};

pub struct Receiver<'a> {
    state: ReceiverState,
    hardware: ReceiverHardware<'a>,
}

impl<'a> Receiver<'a> {
    pub fn new<T, U, V, W>(
        relay_muting_routable_microphone_pin: T,
        relay_routing_routable_microphone_hot_pin: U,
        relay_routing_routable_microphone_cold_pin: V,
        relay_muting_simple_microphone_pin: W,
    ) -> Self
    where
        T: OutputPin + 'a,
        U: OutputPin + 'a,
        V: OutputPin + 'a,
        W: OutputPin + 'a,
    {
        Self {
            state: ReceiverState::default(),
            hardware: ReceiverHardware::new(
                relay_muting_routable_microphone_pin,
                relay_routing_routable_microphone_hot_pin,
                relay_routing_routable_microphone_cold_pin,
                relay_muting_simple_microphone_pin,
            ),
        }
    }

    pub fn initialize(&mut self) {
        // Set all relays to low.
        self.hardware.initialize();

        esp_now::initialize_esp_now_as_receiver();
    }

    pub fn update(&mut self, message: EspNowMessage) {
        // 1. Update state from message.
        self.state.process_message(message);

        // 2. Update the relays from state.
        self.hardware.flush(&self.state);
    }
}

#[derive(Default, Debug)]
struct ReceiverState {
    routable_microphone: RoutableMicrophone,
    simple_microphone: SimpleMicrophone,
}

impl ReceiverState {
    fn process_message(&mut self, message: EspNowMessage) {
        log::info!("process_message: {:?}", message);

        match message {
            EspNowMessage::ResetMicrophone { microphone_type } => {
                self.reset_microphone(microphone_type)
            }
            EspNowMessage::UpdateRoutableMicrophone {
                message_id,
                route,
                active,
            } => self.update_routable_microphone(message_id, route, active),
            EspNowMessage::UpdateSimpleMicrophone { message_id, active } => {
                self.update_simple_microphone(message_id, active)
            }
        };
    }

    /// Reset the specific microphone's `last_message_id`.
    fn reset_microphone(&mut self, microphone_type: MicrophoneType) {
        match microphone_type {
            MicrophoneType::RoutableMicrophone => {
                self.routable_microphone.last_message_id = 0;
            }
            MicrophoneType::SimpleMicrophone => {
                self.simple_microphone.last_message_id = 0;
            }
        }
    }

    /// If `message_id` is valid, update the state of the routable microphone.
    fn update_routable_microphone(
        &mut self,
        message_id: u16,
        route: MicrophoneRoute,
        active: bool,
    ) {
        let microphone = &mut self.routable_microphone;

        if message_id > microphone.last_message_id {
            microphone.route = route;
            microphone.active = active;
            microphone.last_message_id = message_id;
            log::info!("Accepted this message.");
        } else {
            log::warn!("Rejected this old message.");
        }
    }

    /// If `message_id` is valid, update the state of the simple microphone.
    fn update_simple_microphone(&mut self, message_id: u16, active: bool) {
        let microphone = &mut self.simple_microphone;

        if message_id > microphone.last_message_id {
            microphone.active = active;
            microphone.last_message_id = message_id;
            log::info!("Accepted this message.");
        } else {
            log::warn!("Rejected this old message.");
        }
    }
}

struct ReceiverHardware<'a> {
    relay_muting_routable_microphone: PinDriver<'a, Output>,
    relay_routing_routable_microphone_hot: PinDriver<'a, Output>,
    relay_routing_routable_microphone_cold: PinDriver<'a, Output>,
    relay_muting_simple_microphone: PinDriver<'a, Output>,
}

impl<'a> ReceiverHardware<'a> {
    fn new<T, U, V, W>(
        relay_muting_routable_microphone_pin: T,
        relay_routing_routable_microphone_hot_pin: U,
        relay_routing_routable_microphone_cold_pin: V,
        relay_muting_simple_microphone_pin: W,
    ) -> Self
    where
        T: OutputPin + 'a,
        U: OutputPin + 'a,
        V: OutputPin + 'a,
        W: OutputPin + 'a,
    {
        let relay_muting_routable_microphone =
            PinDriver::output(relay_muting_routable_microphone_pin).unwrap();

        let relay_routing_routable_microphone_hot =
            PinDriver::output(relay_routing_routable_microphone_hot_pin).unwrap();

        let relay_routing_routable_microphone_cold =
            PinDriver::output(relay_routing_routable_microphone_cold_pin).unwrap();

        let relay_muting_simple_microphone =
            PinDriver::output(relay_muting_simple_microphone_pin).unwrap();

        Self {
            relay_muting_routable_microphone,
            relay_routing_routable_microphone_hot,
            relay_routing_routable_microphone_cold,
            relay_muting_simple_microphone,
        }
    }

    /// Set all relays to low.
    fn initialize(&mut self) {
        self.relay_muting_routable_microphone.set_low().unwrap();
        self.relay_routing_routable_microphone_hot
            .set_low()
            .unwrap();
        self.relay_routing_routable_microphone_cold
            .set_low()
            .unwrap();
        self.relay_muting_simple_microphone.set_low().unwrap();
    }

    /// Update the relays.
    fn flush(&mut self, state: &ReceiverState) {
        // Mute or unmute the Routable Microphone.
        if state.routable_microphone.active {
            self.relay_muting_routable_microphone.set_high().unwrap();
        } else {
            self.relay_muting_routable_microphone.set_low().unwrap();
        }

        // Change the routing of the Routable Microphone.
        match state.routable_microphone.route {
            MicrophoneRoute::ToAudience => {
                self.relay_routing_routable_microphone_hot
                    .set_low()
                    .unwrap();
                self.relay_routing_routable_microphone_cold
                    .set_low()
                    .unwrap();
            }
            MicrophoneRoute::ToBand => {
                self.relay_routing_routable_microphone_hot
                    .set_high()
                    .unwrap();
                self.relay_routing_routable_microphone_cold
                    .set_high()
                    .unwrap();
            }
        }

        // Mute or unmute the Simple Microphone.
        if state.simple_microphone.active {
            self.relay_muting_simple_microphone.set_high().unwrap();
        } else {
            self.relay_muting_simple_microphone.set_low().unwrap();
        }
    }
}
