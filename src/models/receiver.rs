use crate::models::microphone::{RoutableMicrophone, SimpleMicrophone};

#[derive(Default, Debug)]
pub struct ReceiverState {
    pub routable_microphone: RoutableMicrophone,
    pub simple_microphone: SimpleMicrophone,
}
