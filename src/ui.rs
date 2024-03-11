use crate::*;

/// Button State enum - indicates which buttons are currently pressed.
enum ButtonPressed {
    Neither,
    A,
    B,
    Both,
}

/// Represents the UI state.
struct UiState {
    levels: [u32; 3], // levels for each of the 3 led colors.
    frame_rate: u64,  // frame rate specified for the ui.
}

impl UiState {
    /// Display the current state of the UI - brightness levels for each color and
    /// the frame rate. It prints the information via rtt.
    ///
    /// Iterates over the led color names and their levels and prints them.
    /// Also prints the frame rate.
    fn show(&self) {
        // define the names of the led colors.
        let names = ["red", "green", "blue"];
        rprintln!(); // print a newline.

        // Iterate over the names and corresponding levels together, printing them.
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
    /// The defaults set the levels to the maximum and frame rate to 100.
    ///
    /// # Returns
    ///
    /// A new UiState instance with default values.
    fn default() -> Self {
        Self {
            levels: [LEVELS - 1, LEVELS - 1, LEVELS - 1],
            frame_rate: 100,
        }
    }
}

/// Represents the UI of the program with a knob, A and B buttons and state.
pub struct Ui {
    knob: Knob, // a knob to control the frame rate or brightness of the led.
    button_a: Button, // Button A on the microbit.
    button_b: Button, // Button B on the microbit.
    state: UiState,  // The state of the UI.
}

impl Ui {
    /// Create a new Ui instance with the given arguments. Configure the UiState with
    /// default values. The Knob controls the frame rate, and holding the buttons
    /// changes the control to modify a color brightness level
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
    pub fn new(knob: Knob, button_a: Button, button_b: Button) -> Self {
        Self {
            knob,
            button_a,
            button_b,
            state: UiState::default(),
        }
    }

    fn button_state(&self) -> ButtonPressed {
        let a_pressed = self.button_a.is_low();
        let b_pressed = self.button_b.is_low();

        match (a_pressed, b_pressed) {
            (true, true) => ButtonPressed::Both,
            (true, false) => ButtonPressed::A,
            (false, true) => ButtonPressed::B,
            (false, false) => ButtonPressed::Neither,
        }
    }
    
    /// The main Ui loop, which measures and reports the current values.
    ///
    /// When program starts, it reads the current knob position and updates the
    /// levels accordingly and prints that info to the console. Then it goes into the
    /// main loop which measures, updates and prints the info forever.
    pub async fn run(&mut self) -> ! {
        // Set the level of the green led based on the initial knob position.
        self.state.levels[2] = self.knob.measure().await;

        // Update the global RGB levels var from the measurement.
        set_rgb_levels(|rgb| {
            *rgb = self.state.levels;
        })
        .await;

        // Display the Ui state info.
        self.state.show();

        // Main loop which continuously measures the knob position and 
        // updates the state levels accordingly.
        loop {
            // Measure the knob's current position
            let level = self.knob.measure().await; 

            match self.button_state() {
                ButtonPressed::Both => {
                    rprintln!("Both buttons pressed!");
                },
                ButtonPressed::A => {
                    rprintln!("Button A pressed!");
                },
                ButtonPressed::B => {
                    rprintln!("B button pressed!");

                },
                ButtonPressed::Neither => {
                    rprintln!("Neither button pressed!");
                }
            }

            // If the level has changed...
            if level != self.state.levels[2] {
                // ... update the values and print them.
                self.state.levels[2] = level;
                self.state.show();
                set_rgb_levels(|rgb| {
                    *rgb = self.state.levels;
                })
                .await;
            }

            // Wait for 50 milliseconds.
            Timer::after_millis(50).await;
        }
    }
}
