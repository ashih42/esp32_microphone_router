use esp_idf_hal::gpio::{Input, InputPin, PinDriver, Pull};
use std::time::Instant;
use timed_debouncer::Debouncer;

/// This button sets up a GPIO input pin with internal pull-up resistor, to be used as a momentary pushbutton.
pub struct Button<'a> {
    input: PinDriver<'a, Input>,
    debouncer: Debouncer<bool>,
    start_time: Instant,
    was_pressed: bool,
    on_press_cb: Option<Box<dyn Fn()>>,
    on_release_cb: Option<Box<dyn Fn()>>,
}

const DEFAULT_DEBOUNCE_TICKS: u32 = 20; // 20ms debounce delay

impl<'a> Button<'a> {
    pub fn new<P>(
        pin: P,
        on_press_cb: Option<Box<dyn Fn()>>,
        on_release_cb: Option<Box<dyn Fn()>>,
    ) -> Self
    where
        P: InputPin + 'a,
    {
        Self {
            input: PinDriver::input(pin, Pull::Up).unwrap(),
            debouncer: Debouncer::new(),
            start_time: Instant::now(),
            was_pressed: false,
            on_press_cb,
            on_release_cb,
        }
    }

    /// Check if the button is in a pressed/released state with debouncer, and trigger callbacks on
    /// state-change events.
    fn update(&mut self) {
        let is_pressed = self.debouncer.update(
            self.input.is_low(),
            self.start_time.elapsed().as_millis() as u64,
            DEFAULT_DEBOUNCE_TICKS,
        );

        if !self.was_pressed
            && is_pressed
            && let Some(callback) = &self.on_press_cb
        {
            (callback)();
        } else if self.was_pressed
            && !is_pressed
            && let Some(callback) = &self.on_release_cb
        {
            (callback)();
        }

        self.was_pressed = is_pressed;
    }

    /// Whenever a button is pressed or released, call its callback function.
    pub async fn run(&mut self) {
        loop {
            self.input.wait_for_any_edge().await.unwrap();
            self.update();
        }
    }
}
