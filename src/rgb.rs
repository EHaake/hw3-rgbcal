use crate::*;

/// Type alias for array of AnyPins with static lifetime.
type RgbPins = [Output<'static, AnyPin>; 3];

/// An RGB led is represented here.
pub struct Rgb {
    rgb: RgbPins, // The actual array of pins.
    // Shadow variables to minimize lock contention.
    levels: [u32; 3], // The levels for each pin as an array of u32.
    tick_time: u64,   // The duration of a tick as a u64.
}

impl Rgb {
    /// Calculates the frame tick time from a given frame rate in microseconds.
    ///
    /// # Arguments
    ///
    /// * 'frame_rate' - a u64 that specifies a frame rate.
    ///
    /// # Returns
    ///
    /// The tick time in microseconds of a frame as a u64.
    fn frame_tick_time(frame_rate: u64) -> u64 {
        // divide from 1000000 to convert to microseconds.
        1_000_000 / (3 * frame_rate * LEVELS as u64)
    }

    /// Creates a new 'Rgb' instance with the given pins and frame rate.
    ///
    /// # Arguments
    ///
    /// * 'rgb' - The array of RgbPins.
    /// * 'frame_rate' - the frame rate in fps for updating the led.
    ///
    /// # Returns
    ///
    /// A new 'Rgb' instance.
    pub fn new(rgb: RgbPins, frame_rate: u64) -> Self {
        // calculate the tick_time from the frame_rate.
        let tick_time = Self::frame_tick_time(frame_rate);
        // return a new struct instance, setting initial levels to 0.
        Self {
            rgb,
            levels: [0; 3], 
            tick_time,
        }
    }

    /// Performs an on/off 'step' for a single, specified led.
    ///
    /// Turns on the LED for a duration in proportion to it's frame rate
    /// and level then turns it off for the rest of the frame period.
    ///
    /// # Arguments
    ///
    /// * 'led' - a usize indicating which led to step.
    async fn step(&mut self, led: usize) {
        // Get the current brightness for the specified led.
        let level = self.levels[led];

        // Turn on led for a period of time if level is non-zero.
        if level > 0 {
            // Turn on led.
            self.rgb[led].set_high(); 

            // Calculate the time the led should be on for based on the current level,
            // and the tick time.
            let on_time = level as u64 * self.tick_time; 

            // Wait for the specified time in microseconds.
            Timer::after_micros(on_time).await;

            // Turn off the led.
            self.rgb[led].set_low();
        }

        // Calculate the new level for the off period.
        let level = LEVELS - level;

        // If level is still non-zero...
        if level > 0 {
            // Calculate the time for the off period.
            let off_time = level as u64 * self.tick_time;
            
            // Wait for specified time in microseconds.
            Timer::after_micros(off_time).await;
        }
    }

    /// Continuously update the brightness of each led.
    ///
    /// This function runs forever and so should never exit.
    pub async fn run(mut self) -> ! {
        loop {
            // Get the current brightness levels for all leds
            // and update internal value.
            self.levels = get_rgb_levels().await;

            // Get the frame rate and calculate the tick time from it,
            // updating the internal value.
            let frame_rate = get_frame_rate().await;
            self.tick_time = Self::frame_tick_time(frame_rate);

            // Update brightness of each led in sequence.
            for led in 0..3 {
                self.step(led).await;
            }
        }
    }
}
