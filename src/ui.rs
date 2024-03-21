use crate::*;

/// Button State enum - Available button states.
enum ButtonPressed {
    Neither,
    A,
    B,
    Both,
}

/// Control Enum - Available controls to modify.
enum Control {
    RedLed,
    GreenLed,
    BlueLed,
    FrameRate,
}

/// Represents the UI state.
struct UiState {
    levels: [u32; 3], // levels for each of the 3 led colors.
    frame_rate: u64,  // frame rate specified for the ui.
}

impl UiState {
    /// Display the current state of the UI - brightness levels for each color
    /// and the frame rate. It prints the information via rtt.
    ///
    /// Iterates over the led color names and their levels and prints them.
    /// Also prints the frame rate.
    fn show(&self) {
        // define the names of the led colors.
        let names = ["red", "green", "blue"];
        rprintln!(); // print a newline.

        // Iterate over the names and levels together, printing them.
        for (name, level) in names.iter().zip(self.levels.iter()) {
            rprintln!("{}: {}", name, level);
        }
        // Also print the frame rate.
        rprintln!("frame rate: {}", self.frame_rate);
    }
}

impl Default for UiState {
    /// Default values for an instance of UiState.
    ///
    /// The defaults set the levels determined to be 'white' on my board.
    ///
    /// # Returns
    ///
    /// A new UiState instance with default values.
    fn default() -> Self {
        Self {
            levels: [15, 4, 6],
            frame_rate: 100,
        }
    }
}

/// Represents the UI of the program with a knob, A and B buttons and state.
pub struct Ui {
    knob: Knob,             // a knob to control the frame rate or brightness.
    button_a: Button,       // Button A on the microbit.
    button_b: Button,       // Button B on the microbit.
    levelmeter: LevelMeter, // Level Meter
    state: UiState,         // The state of the UI.
}

impl Ui {
    /// Create a new Ui instance with the given arguments. Configure the
    /// UiState with default values. The Knob controls the frame rate,
    /// and holding the buttons changes the control to modify a color
    /// brightness level.
    ///
    /// # Arguments
    ///
    /// * 'knob' - The control for modifying brightness settings.
    /// * 'button_a' - The A button on the Microbit.
    /// * 'button_b' - The B button on the Microbit.
    ///
    /// # Returns
    ///
    /// A new 'Ui' instance.
    pub fn new(knob: Knob, button_a: Button, button_b: Button, levelmeter: LevelMeter) -> Self {
        Self {
            knob,
            button_a,
            button_b,
            levelmeter,
            state: UiState::default(),
        }
    }

    /// Figures out which combination of buttons is being pressed and then
    /// returns the appropriate enum value.
    ///
    /// # Returns
    ///
    /// A 'ButtonPressed' enum value correspoding to which buttons are pressed.
    fn button_state(&self) -> ButtonPressed {
        let a_pressed = self.button_a.is_low(); // check if button a is pressed.
        let b_pressed = self.button_b.is_low(); // check if button b is pressed.

        // Match the state of buttons pressed and return the appropriate value.
        match (a_pressed, b_pressed) {
            (true, true) => ButtonPressed::Both,
            (true, false) => ButtonPressed::A,
            (false, true) => ButtonPressed::B,
            (false, false) => ButtonPressed::Neither,
        }
    }

    /// Scale the knob level to a value from 10..160 in steps of 10 to
    /// calculate the frame rate.
    ///
    /// # Arguments
    ///
    /// * 'level' - The measured level from the knob scaled from 0..15 as a u32.
    ///
    /// # Returns
    ///
    /// A scaled level to be used as a frame rate as a u64.
    fn frame_rate_from_level(&self, level: u32) -> u64 {
        let scaled_level = (level + 1) * 10;
        scaled_level as u64
    }

    /// The main Ui loop, which measures and reports the current values.
    ///
    /// When program starts, it reads the current knob position and updates the
    /// levels accordingly and prints that info to the console.
    /// Then it goes into the main loop which measures,
    /// updates and prints the info forever.
    pub async fn run(&mut self) -> ! {
        // Display the Ui state info.
        self.state.show();

        // Main loop which continuously measures the knob position and
        // updates the state levels accordingly.
        loop {
            // Measure the knob's current position
            let level = self.knob.measure().await;

            // Flag to indicate if a level has been changed.
            let mut control_changed = false;

            // Choose the appropriate control to modify based on which buttons
            // are being pressed.
            let control = match self.button_state() {
                ButtonPressed::Both => Control::RedLed,
                ButtonPressed::A => Control::BlueLed,
                ButtonPressed::B => Control::GreenLed,
                ButtonPressed::Neither => Control::FrameRate,
            };

            // Adjust the led color corresponding to the control selected.
            match control {
                Control::RedLed => {
                    if level != self.state.levels[0] {
                        self.state.levels[0] = level;
                        control_changed = true;
                    }
                }
                Control::GreenLed => {
                    if level != self.state.levels[1] {
                        self.state.levels[1] = level;
                        control_changed = true;
                    }
                }
                Control::BlueLed => {
                    if level != self.state.levels[2] {
                        self.state.levels[2] = level;
                        control_changed = true;
                    }
                }
                Control::FrameRate => {
                    let frame_rate = self.frame_rate_from_level(level);
                    if frame_rate != self.state.frame_rate {
                        self.state.frame_rate = frame_rate;
                        control_changed = true;
                    }
                }
            }

            // Display and update the new values only if a change has occurred.
            if control_changed {
                // Print the current state.
                self.state.show();

                // Update the global rgb levels Mutex.
                set_rgb_levels(|rgb| {
                    *rgb = self.state.levels;
                })
                .await;

                // Update the global frame_rate level Mutex
                set_frame_rate(|frame_rate| {
                    *frame_rate = self.state.frame_rate;
                })
                .await;
            }

            // Update the levelmeter every step.
            // This adds a 50ms delay to avoid overmeasuring
            // the knob level.
            self.levelmeter.update_display(self.state.levels).await;
        }
    }
}
