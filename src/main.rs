#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_sync::{
    mutex::Mutex,
    blocking_mutex::raw::ThreadModeRawMutex,
};
use embassy_futures::join;
use microbit_bsp::{
    embassy_nrf::{
        bind_interrupts,
        gpio::{AnyPin, Level, Output, OutputDrive},
        saadc,
    },
    Microbit,
};
use panic_probe as _;

type Adc = saadc::Saadc<'static, 1>;

struct Knob(Adc);

impl Knob {
    async fn new(adc: Adc) -> Self {
        adc.calibrate().await;
        Self(adc)
    }

    async fn _measure(&mut self) -> f32 {
        let mut buf = [0];
        self.0.sample(&mut buf).await;
        buf[0].clamp(0, 0x7fff) as f32 / 0x7fff as f32
    }
}

type RgbPins = [Output<'static, AnyPin>; 3];

static RGB_LEVELS: Mutex<ThreadModeRawMutex, [u32; 3]> = Mutex::new([0; 3]);
const LEVELS: u32 = 8;

async fn get_rgb_levels() -> [u32; 3] {
    let rgb_levels = RGB_LEVELS.lock().await;
    *rgb_levels
}

async fn _set_rgb_levels<F>(setter: F)
    where F: FnOnce(&mut [u32; 3])
{
    let mut rgb_levels = RGB_LEVELS.lock().await;
    setter(&mut *rgb_levels);
}

struct Rgb {
    rgb: RgbPins,
    // Shadow array to minimize lock contention.
    levels: [u32; 3],
    tick: u32,
    tick_time: Duration,
}

impl Rgb {
    fn frame_tick_time(frame_rate: u64) -> Duration {
        Duration::from_micros(1_000_000 / (3 * frame_rate * LEVELS as u64))
    }

    fn new(rgb: RgbPins, frame_rate: u64) -> Self {
        let tick_time = Self::frame_tick_time(frame_rate);
        Self { rgb, levels: [0; 3], tick: 0, tick_time }
    }

    async fn step(&mut self) {
        let led = self.tick / LEVELS;
        let level = self.tick % LEVELS;
        if level == 0 {
            if led == 0 {
                self.levels = get_rgb_levels().await;
            }

            let prev = (led + 2) % 3;
            if self.rgb[prev as usize].is_set_high() {
                self.rgb[prev as usize].set_low();
            }
        }
        if level < self.levels[led as usize] {
            self.rgb[led as usize].set_high();
        } else if self.rgb[led as usize].is_set_high() {
            self.rgb[led as usize].set_low();
        }
        self.tick = (self.tick + 1) % (3 * LEVELS);
        Timer::after(self.tick_time).await;
    }
}

async fn run_rgb(mut rgb: Rgb) -> ! {
    loop {
        rgb.step().await;
    }
}

async fn run_ui(_knob: Knob) -> ! {
    todo!()
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let board = Microbit::default();

    bind_interrupts!(struct Irqs {
        SAADC => saadc::InterruptHandler;
    });

    let led_pin = |p| Output::new(p, Level::Low, OutputDrive::Standard);
    let red = led_pin(AnyPin::from(board.p9));
    let green = led_pin(AnyPin::from(board.p8));
    let blue = led_pin(AnyPin::from(board.p16));
    let rgb: Rgb = Rgb::new([red, green, blue], 100);

    let mut saadc_config = saadc::Config::default();
    saadc_config.resolution = saadc::Resolution::_14BIT;
    let saadc = saadc::Saadc::new(
        board.saadc,
        Irqs,
        saadc_config,
        [saadc::ChannelConfig::single_ended(board.p2)],
    );
    let knob = Knob::new(saadc).await;

    join::join(
        run_rgb(rgb),
        run_ui(knob),
    ).await;

    panic!("fell off end of main loop");
}