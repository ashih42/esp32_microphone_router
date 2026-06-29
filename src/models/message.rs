use crate::models::microphone::MicrophoneRoute;
use std::sync::atomic::{AtomicU16, Ordering};

/// `message_id` is an auto-increment value for receiver to determine if messages were received out of order.
/// u16 provides a sufficient range of values for real application.
/// (Suppose if one message is sent every second, it would still take 18 hours to reach u16 max value.)
///
/// Since there are only 2 microphones, `route` is sufficient to determine which microphone this message is for.
#[derive(Debug)]
#[repr(C)]
pub struct Message {
    pub message_id: u16,
    pub route: Option<MicrophoneRoute>,
    pub active: bool,
}

pub const MESSAGE_SIZE: usize = std::mem::size_of::<Message>();

pub trait ToMessage {
    fn to_message(&self) -> Message;
}

impl Message {
    pub fn generate_message_id() -> u16 {
        static COUNTER: AtomicU16 = AtomicU16::new(1);

        // Return the current value in `COUNTER`, and then increment `COUNTER`.
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
}
