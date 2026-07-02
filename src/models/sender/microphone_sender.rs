pub trait MicrophoneSender {
    fn initialize(&mut self);
    fn update(&mut self);
}
