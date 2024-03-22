use embassy_time::Duration;
use microbit_bsp::display::{Brightness, Frame};

use crate::*;

/// Represents a LevelMeter for the r,g,b values from the
/// 5x5 LED grid on the Microbit::v2.
pub struct LevelMeter {
    display: LedMatrix,
}

impl LevelMeter {
    /// Creates a new LevelMeter.
    ///
    /// # Arguments
    ///
    /// * 'display' - The 5x5 LedMatrix on the Microbit::v2.
    pub fn new(mut display: LedMatrix) -> Self {
        // display.clear();
        display.set_brightness(Brightness::MAX);
        Self { display }
    }

    /// Update the level meter display based on the current rgb levels.
    /// Async fn so that we can keep the 50ms pause in measuring the
    /// knob level from the ui loop.
    ///
    /// # Arguments
    ///
    /// * 'levels' - The current red, green and blue levels as an array of u32s.
    pub async fn update_display(&mut self, levels: [u32; 3], frame_rate: u64) {
        // Create a new frame object.
        let mut frame = Frame::default();

        // Iterate through the levels and set the appropriate leds in the matrix.
        for (i, &level) in levels.iter().enumerate() {
            // Determine the correct col for each of the rgb channels
            // 1st col -> red, 2nd col -> green, 3rd col -> blue, 5th col -> framerate
            let col = i;

            // Decide how many leds to light up based on its level.
            // Since we have 16 levels but only 5 leds (and off), we need to scale.
            let lit_leds = ((level as f32) * 5.0 / LEVELS as f32).ceil() as usize;

            // Light up the appropriate number of leds in the column for that color.
            for idx in 0..lit_leds {
                // Calculate the actual row index for the led, to start from bottom.
                let actual_led_row = 4 - idx;

                // Set the led in the frame buffer.
                frame.set(col, actual_led_row);
            }
        }

        // The frame rate is on a different scale so we need to normalize differently.
        let frame_rate_leds = ((frame_rate as f32 - 10.0) * 5.0 / (160.0 - 10.0)).ceil() as usize;

        // Use column index 4 (5th column) for the frame rate.
        for idx in 0..frame_rate_leds {
            let actual_led_row = 4 - idx;
            frame.set(4, actual_led_row);
        }

        // Display the frame for 50ms.
        // This is here in place of the 50ms delay at the end of the ui loop.
        self.display.display(frame, Duration::from_millis(50)).await;
    }
}
