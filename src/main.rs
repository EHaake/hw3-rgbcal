#![no_std]
#![no_main]

mod knob;
mod rgb;
mod ui;
pub use knob::*;
pub use rgb::*;
pub use ui::*;

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use embassy_executor::Spawner;
use embassy_futures::join;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::Timer;
use microbit_bsp::{
    embassy_nrf::{
        bind_interrupts,
        gpio::{AnyPin, Level, Output, OutputDrive},
        saadc,
    },
    Button, Microbit,
};
use num_traits::float::FloatCore;

/// Global value to store the current RGB values.
pub static RGB_LEVELS: Mutex<ThreadModeRawMutex, [u32; 3]> = Mutex::new([0; 3]);
/// Global value to store the current FRAME RATE value
pub static FRAME_RATE: Mutex<ThreadModeRawMutex, u64> = Mutex::new(100);
/// 16 levels for each RGB value.
pub const LEVELS: u32 = 16;

/// Returns the current rgb values from the global Mutex asynchronously.
///
/// Acquires a lock on the global RGB_LEVELS var for the duration of the fn.
///
/// # Returns
///
/// The current rgb values as an array of 3 u32 values.
async fn get_rgb_levels() -> [u32; 3] {
    let rgb_levels = RGB_LEVELS.lock().await;
    *rgb_levels
}

/// Sets the current RGB values into the Mutex asynchronously.
///
/// # Arguments
/// * 'setter' - A setter function as a closure that can only be called once
/// with a mutable reference to the RGB values as a mutable array of 3 u32s.
async fn set_rgb_levels<F>(setter: F)
where
    F: FnOnce(&mut [u32; 3]),
{
    // Get a lock on the RGB_LEVELS Mutex.
    let mut rgb_levels = RGB_LEVELS.lock().await;
    // Set the values with the provided setter fn.
    setter(&mut rgb_levels);
}

/// Returns the current frame rate value from a global Mutex asynchronously.
///
/// Acquires a lock on the global FRAME_RATE var for the duration of the fn.
///
/// # Returns
///
/// The current value in FRAME_RATE as a u64.
async fn get_frame_rate() -> u64 {
    let frame_rate = FRAME_RATE.lock().await;
    *frame_rate
}

/// Sets the current frame rate value into a global Mutex asynchronously.
///
/// # Arguments
/// * 'setter' - A setter function as a closure that can only be called once
/// with a mutable reference to the frame rate value as a u64.
async fn set_frame_rate<F>(setter: F)
where
    F: FnOnce(&mut u64),
{
    let mut frame_rate = FRAME_RATE.lock().await;
    setter(&mut frame_rate);
}

/// Main function - is async and doesn't return.
#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    rtt_init_print!(); // Initialize rtt debug printing.
    let board = Microbit::default(); // Initialize the board with defaults.

    // Setup the interrupt handler for the SAADC peripheral.
    bind_interrupts!(struct Irqs {
        SAADC => saadc::InterruptHandler;
    });

    // Define a closure that helps to configure the individual rgb leds.
    // Takes a pin, and sets the level to low and drive to standard.
    let led_pin = |p| Output::new(p, Level::Low, OutputDrive::Standard);

    // Setup and configure the individual rgb led pins.
    let red = led_pin(AnyPin::from(board.p9));
    let green = led_pin(AnyPin::from(board.p8));
    let blue = led_pin(AnyPin::from(board.p16));

    // Group the led pins into an RGB struct with a specified frame rate.
    let rgb: Rgb = Rgb::new([red, green, blue], 100);

    // Configure the SAADC with defaults, then set resolution to 14 bit.
    let mut saadc_config = saadc::Config::default();
    saadc_config.resolution = saadc::Resolution::_14BIT;

    // Initialize the SAADC with configuration and connect to pin 2.
    let saadc = saadc::Saadc::new(
        board.saadc,
        Irqs,
        saadc_config,
        [saadc::ChannelConfig::single_ended(board.p2)],
    );

    // Initialize the knob interface with the initialized SAADC.
    let knob = Knob::new(saadc).await;

    // Initialize the UI interface with the knob, and a,b board buttons.
    let mut ui = Ui::new(knob, board.btn_a, board.btn_b);

    // This is the main loop -
    // Run the rgb and ui loops concurrently by joining them.
    join::join(rgb.run(), ui.run()).await;

    // Panic and print this message if the above loops ever exit.
    panic!("fell off end of main loop");
}
