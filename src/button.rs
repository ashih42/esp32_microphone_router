// use esp_hal::{
//     // delay::Delay,
//     gpio::{Input, InputConfig, InputPin, Level, Output, OutputConfig, Pull},
//     time::{Duration, Instant},
//     Config,
// };

use std::time::Instant;

use esp_idf_hal::gpio::{Input, InputPin, PinDriver, Pull};
use timed_debouncer::Debouncer;

/// This button sets up a GPIO input pin with internal pull-up resistor, to be used as a momentary pushbutton.
pub struct Button<'a> {
    input: PinDriver<'a, Input>,
    debouncer: Debouncer<bool>,
    start_time: Instant,
    was_pressed: bool,
    on_press_cb: Box<dyn FnMut()>,
    on_release_cb: Box<dyn FnMut()>,
}

const DEFAULT_DEBOUNCE_TICKS: u32 = 20; // 20ms debounce delay

impl<'a> Button<'a> {
    pub fn new<P>(pin: P, on_press_cb: Box<dyn FnMut()>, on_release_cb: Box<dyn FnMut()>) -> Self
    where
        P: InputPin + 'a,
    {
        let pin_number = pin.pin();

        let input = PinDriver::input(pin, Pull::Up)
            .unwrap_or_else(|_| panic!("Could not set up button on pin {}", pin_number));

        Self {
            input,
            debouncer: Debouncer::new(),
            start_time: Instant::now(),
            was_pressed: false,
            on_press_cb,
            on_release_cb,
        }
    }

    /// Check if the button is in a pressed/released state with debouncer, and trigger callbacks on
    /// state-change events.
    pub fn update(&mut self) {
        let is_pressed = self.debouncer.update(
            self.input.is_low(),
            self.start_time.elapsed().as_millis() as u64,
            DEFAULT_DEBOUNCE_TICKS,
        );

        if !self.was_pressed && is_pressed {
            (self.on_press_cb)();
        } else if self.was_pressed && !is_pressed {
            (self.on_release_cb)();
        }

        self.was_pressed = is_pressed;
    }

    pub async fn wait_and_update(&mut self) {
        self.input.wait_for_any_edge().await.unwrap();
        self.update();
    }
}
