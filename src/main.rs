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

/// Global value to store the current RGB values
pub static RGB_LEVELS: Mutex<ThreadModeRawMutex, [u32; 3]> = Mutex::new([0; 3]);
/// 16 levels for each RGB value
pub const LEVELS: u32 = 16;

/// Returns the current rgb values from the global Mutex asynchronously
///
/// Acquires a lock on the global RGB_LEVELS variable for the duration of the fn.
async fn get_rgb_levels() -> [u32; 3] {
    let rgb_levels = RGB_LEVELS.lock().await;
    *rgb_levels
}

/// Sets the current RGB values into the Mutex asynchronously
///
/// Takes a setter function as a closure that can only be called once with
/// a mutable reference to the RGB values as a mutable array of 3 u32 values.
async fn set_rgb_levels<F>(setter: F)
where
    F: FnOnce(&mut [u32; 3]),
{
    let mut rgb_levels = RGB_LEVELS.lock().await;
    setter(&mut rgb_levels);
}

/// Main function - is async and doesn't return
#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    rtt_init_print!(); // initialize rtt debug printing
    let board = Microbit::default(); // initialize the board with defaults

    // setup the interrupt handler for the SAADC peripheral
    bind_interrupts!(struct Irqs {
        SAADC => saadc::InterruptHandler;
    });

    // define a closure that helps to configure the individual rgb leds 
    // takes a pin, and sets the level to low and drive to standard
    let led_pin = |p| Output::new(p, Level::Low, OutputDrive::Standard);

    // setup and configure the individual rgb led pins
    let red = led_pin(AnyPin::from(board.p9));
    let green = led_pin(AnyPin::from(board.p8));
    let blue = led_pin(AnyPin::from(board.p16));

    // group the led pins into an RGB struct with a specified frame rate
    let rgb: Rgb = Rgb::new([red, green, blue], 100);

    // configure the SAADC with defaults, then set resolution to 14 bit
    let mut saadc_config = saadc::Config::default();
    saadc_config.resolution = saadc::Resolution::_14BIT;

    // initialize the SAADC with configuration and connect to pin 2
    let saadc = saadc::Saadc::new(
        board.saadc,
        Irqs,
        saadc_config,
        [saadc::ChannelConfig::single_ended(board.p2)],
    );

    // initialize the knob interface with the initialized SAADC
    let knob = Knob::new(saadc).await;

    // initialize the UI interface with the knob, and a,b board buttons
    let mut ui = Ui::new(knob, board.btn_a, board.btn_b);

    // Run the rgb and ui loops concurrently by joining them
    // This is the main loop
    join::join(rgb.run(), ui.run()).await;

    // Panic and print this message if the above loops ever exit
    panic!("fell off end of main loop");
}
