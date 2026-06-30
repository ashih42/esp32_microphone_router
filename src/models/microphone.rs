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

#[repr(u8)]
#[derive(Default, Debug, Clone, Copy)]
pub enum MicrophoneRoute {
    #[default]
    ToAudience,
    ToBand,
}
