use embassy_time::Duration;
use microbit_bsp::display::{Brightness, Frame};

use crate::*;

// const XSIZE: usize = 5;
// const YSIZE: usize = 5;

/// Represents a LevelMeter for the r,g,b values from the
/// 5x5 LED grid on the Microbit::v2
pub struct LevelMeter {
    display: LedMatrix,
    // frame: Frame<XSIZE, YSIZE>,
}

impl LevelMeter {
    /// Creates a new LevelMeter
    ///
    /// # Arguments
    ///
    /// * 'display' - The 5x5 LedMatrix on the Microbit::v2
    pub fn new(mut display: LedMatrix) -> Self {
        // display.clear();
        display.set_brightness(Brightness::MAX);
        // Self { display, frame: Frame::default() }
        Self { display }
    }

    /// Update the level meter display based on the current rgb levels
    ///
    /// # Arguments
    ///
    /// * 'levels' - The current red, green and blue levels as an array of u32s
    pub async fn update_display(&mut self, levels: [u32; 3]) {
        let mut frame = Frame::default();

        for (i, &level) in levels.iter().enumerate() {
            let row = i * 2;

            let lit_cols = ((level as f32) * 5.0 / LEVELS as f32).ceil() as usize;

            for col in 0..lit_cols {
                let actual_col = 4 - col;
                frame.set(row, actual_col);
                // self.display.on(row, actual_col);
            }
        }

        // self.display.render();
        self.display.display(frame, Duration::from_millis(10)).await;
        // self.display.apply(frame);
    }
}
