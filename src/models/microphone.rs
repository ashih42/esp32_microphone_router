#[derive(Default, Debug)]
pub struct RoutableMicrophone {
    pub route: MicrophoneRoute,
    pub active: bool,
    pub last_message_id: u16,
}

#[derive(Default, Debug)]
pub struct SimpleMicrophone {
    pub active: bool,
    pub last_message_id: u16,
}

#[derive(Default, Debug)]
#[repr(u8)]
pub enum MicrophoneRoute {
    #[default]
    ToAudience,
    ToBand,
}
