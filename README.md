# rgbcal: RGB LED calibration tool
Bart Massey 2024  
Forked by Erik Haake 2024

This tool is designed to find out a decent frame rate and
maximum RGB component values to produce a white-looking RGB
of reasonable brightness.

See below for UI.

**This tool is finished!**

Values for what appears to be as 'white' as possible with the provided board 
and led are as follows:

- Red: 15
- Green: 4
- Blue: 6
- Frame Rate: 100

The program starts with the defaults and 'should' appear white. Touching any
of the buttons or knob will set the corresponding setting to the value
measured at the knob.

## Build and Run

Run with `cargo embed --release`. You'll need `cargo embed`, as
`cargo run` / `probe-rs run` does not reliably maintain a
connection for printing. See
https://github.com/probe-rs/probe-rs/issues/1235 for the
details.

## Wiring

Connect the RGB LED to the MB2 as follows:

* Red to P9 (GPIO1)
* Green to P8 (GPIO2)
* Blue to P16 (GPIO3)
* Gnd to Gnd

Connect the potentiometer (knob) to the MB2 as follows:

* Pin 1 to Gnd
* Pin 2 to P2
* Pin 3 to +3.3V

## UI

The knob controls the individual settings: frame rate and
color levels. Which parameter the knob controls should be
determined by which buttons are held. (Right now, the knob
jus always controls Blue. You should see the color change
from green to teal-blue as you turn the knob clockwise.)

* No buttons held: Change the frame rate in steps of 10
  frames per second from 10..160.
* A button held: Change the blue level from off to on over
  16 steps.
* B button held: Change the green level from off to on over
  16 steps.
* A+B buttons held: Change the red level from off to on over
  16 steps.

The "frame rate" (also known as the "refresh rate") is the
time to scan out all three colors. (See the scanout code.)
At 30 frames per second, every 1/30th of a second the LED
should scan out all three colors. If the frame rate is too
low, the LED will appear to "blink". If it is too high, it
will eat CPU for no reason.

I think the frame rate is probably set higher than it needs
to be right now: it can be tuned lower.

## Erik's Comments

First I went through each file and added documentation to very function and most lines. This was helpful to understand what every part of the code was doing, as well as to practice writing doc comments in proper Rust style. To figure out how to write correctly styled doc comments, I looked in the Rust by Example section on it [here](https://doc.rust-lang.org/rust-by-example/meta/doc.html).

Since the starting configuration is that the knob controls the blue level, I next needed to enable the buttons. The buttons were already included in the Ui, so it was a simple matter to get them configured. I added a `ButtonPressed` enum and an implementation in `Ui` that reads the button states and returns a `ButtonPressed` value. Then, in the main `Ui` loop, I can match over the enum and call the appropriate code. This was inspired by an earlier demo in which we try to press both buttons at the same time. 

In order to get the appropriate led color level to be adjusted by the position of the knob and which button(s) is/are pressed, I added another enum `Control` which lets me do another match. I first match over the `button_state` to determine which `control` to adjust and then match over the control and modify the appropriate index in `self.state.levels`. I also added a flag to indicate whether there was a change made, so that I'm not constantly printing the status.

Implenting the frame rate adjustment took a little bit more work. I figured I would need another global Mutex `FRAME_RATE` similar to the one for `RGB_LEVELS` so I put that in as well as a setter and getter for it.

Next, in the `Ui` I needed to scale the frame rate since the knob's `measure()` function returns a value scaled from 0 to 15. For that I implemented a simple scaling function on `Ui` which does just that and returns the value between 10 and 160 in steps of 10 as a u64, just as the spec requires. Then in  the `Control` match statement I compate the `frame_rate` value in the `Ui`'s  `UiState` struct. Like the other controls, if the current state doesn't match the internal state, I update the `Ui` and set a flag to update the global Mutex.

Finally, in the `run()` function in `Rgb` I just need to get the frame rate from the global Mutex, calculate the `frame_tick_time` from it and set `self.tick_time` to that value. That's all I need to do since `step()` uses the internal `tick_time` value already to turn each LED on and off for the appropriate amount of time.

One last thing was I set the `led_pin` `OutputDrive` to `HighDrive`. With the default of `Standard`,I couldn't quite get it to look white. Changing this made a noticeable difference to my eyes in terms of how white the led became.

To finish up, I set hard coded the values I determined to be the most 'white' looking the that I could get from the LED to be the default settings. When starting program with everything wired up correctly, the led should appear white, and then touching any of the controls will then mess with the settings as expected.
