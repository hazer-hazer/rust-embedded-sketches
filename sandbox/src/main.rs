//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use cortex_m::singleton;
use defmt::*;
use embedded_hal::{digital::v2::OutputPin, Pwm};

extern crate sandbox;

use heapless::spsc::Queue;
use num::{pow::Pow, Integer, ToPrimitive};
use pio_proc::pio_asm;
use sandbox::{
    audio::{
        osc::simple_form::{SimpleFormSource, WaveForm},
        source::Sample,
    },
    bsp,
    dsp::math::{F32Ext, PI2},
    exit,
    heap::init_global_heap,
    rp2040_ext::pio_i2s::PioI2S,
    Vec,
};

use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{
        ascii::{FONT_4X6, FONT_5X7, FONT_7X13},
        MonoFont,
    },
    pixelcolor::{raw::ToBytes, Rgb565, RgbColor},
    primitives::{PrimitiveStyleBuilder, Rectangle, StyledDrawable},
};
use embedded_graphics_framebuf::FrameBuf;

use sandbox::{
    display::fps::FPS,
    dsp::{
        math::SampleList,
        resampler::{Resampler, ResamplerBuilder},
        waves::shapes::sine_wave,
    },
    fmt::FmtBuf,
};
use st7735_lcd;

use embedded_graphics::{
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use static_cell::StaticCell;

use micromath::F32Ext as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
// use rp_pico as bsp;
// use waveshare_rp2040_zero as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::{
    entry,
    hal::{
        self,
        clocks::{ClocksManager, InitError},
        dma::{DMAExt, Pace},
        fugit::{HertzU32, Instant},
        gpio::{FunctionPio0, FunctionPio1},
        pio::{PIOBuilder, PIOExt, PinDir},
        pll::{common_configs::PLL_USB_48MHZ, setup_pll_blocking, PLLConfig},
        xosc::setup_xosc_blocking,
    },
    XOSC_CRYSTAL_FREQ,
};

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    fugit::RateExtU32,
    pac,
    pwm::{self},
    sio::Sio,
    watchdog::Watchdog,
    Spi,
};

const DISPLAY_WIDTH: u32 = 160;
const DISPLAY_HEIGHT: u32 = 80;

// const DISPLAY_WIDTH: u32 = 240;
// const DISPLAY_HEIGHT: u32 = 240;

// const MAX_SYS_CLOCK: u32 = 125_000_000;
const I2S_SAMPLE_RATE: u32 = 48000;
// const I2S_BIT_DEPTH: u32 = 32;
// const SYS_PLL_DIVS: (u8, u8) = (5, 2);
const SYS_CLOCK_FREQ: HertzU32 = HertzU32::Hz(153_600_000);

// fn pickup_best_sys_clock() {
//     let max_fitting = 0;
//     for i in 1..100 {
//         let freq = I2S_SAMPLE_RATE * 64 * i;
//         if freq <= MAX_SYS_CLOCK {
//             max_fitting = freq;
//         }
//     }
//     return max_fitting;
// }

// const fn calc_sys_clock() -> HertzU32 {
//     let max_less_than_125 = 0;
//     for i in 10..100 {
//         let freq =
//             I2S_SAMPLE_RATE * I2S_BIT_DEPTH * 2 * SYS_PLL_DIVS.0 as u32 * SYS_PLL_DIVS.1 as u32;
//         if freq <= 125_000_000 {
//             max_less_than_125 = freq;
//         } else {
//             break;
//         }
//     }
//     defmt::assert!(max_less_than_125 > 0 && max_less_than_125 <= 125_000_000);
//     max_less_than_125.Hz()
// }

#[entry]
fn main() -> ! {
    info!("Program start");

    unsafe { init_global_heap() };

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    watchdog.enable_tick_generation((XOSC_CRYSTAL_FREQ / 1_000_000) as u8);

    let sio = Sio::new(pac.SIO);

    // let clocks = init_clocks_and_plls(
    //     external_xtal_freq_hz,
    //     pac.XOSC,
    //     pac.CLOCKS,
    //     pac.PLL_SYS,
    //     pac.PLL_USB,
    //     &mut pac.RESETS,
    //     &mut watchdog,
    // )
    // .ok()
    // .unwrap();

    let mut clocks = ClocksManager::new(pac.CLOCKS);
    let xosc = setup_xosc_blocking(pac.XOSC, XOSC_CRYSTAL_FREQ.Hz()).unwrap();

    {
        let pll_sys = setup_pll_blocking(
            pac.PLL_SYS,
            xosc.operating_frequency(),
            PLLConfig {
                vco_freq: SYS_CLOCK_FREQ * 5,
                refdiv: 1,
                post_div1: 5,
                post_div2: 1,
            },
            &mut clocks,
            &mut pac.RESETS,
        )
        .map_err(InitError::PllError)
        .ok()
        .unwrap();

        let pll_usb = setup_pll_blocking(
            pac.PLL_USB,
            xosc.operating_frequency(),
            PLL_USB_48MHZ,
            &mut clocks,
            &mut pac.RESETS,
        )
        .map_err(InitError::PllError)
        .ok()
        .unwrap();

        clocks.init_default(&xosc, &pll_sys, &pll_usb).unwrap();

        debug!("System clock: {}MHz", clocks.system_clock.freq().to_MHz());
        debug!(
            "Peripheral clock: {}MHz",
            clocks.peripheral_clock.freq().to_MHz()
        );
    }

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // let mut led = pins.led.into_push_pull_output();

    let dma = pac.DMA.split(&mut pac.RESETS);

    // info!("DMA Clock: {}", dma.cl);

    // let mut i2s = PioI2S::new(
    //     pac.PIO0,
    //     (dma.ch0, dma.ch1),
    //     &mut pac.RESETS,
    //     I2S_SAMPLE_RATE,
    //     &clocks,
    //     // pins.gpio21.reconfigure(),
    //     pins.gpio18.reconfigure(),
    //     pins.gpio19.reconfigure(),
    //     pins.gpio20.reconfigure(),
    // );

    let mut i2s = PioI2S::new(
        pac.PIO0,
        (dma.ch0, dma.ch1),
        &mut pac.RESETS,
        I2S_SAMPLE_RATE,
        &clocks,
        pins.gpio18.reconfigure(),
        pins.gpio19.reconfigure(),
        pins.gpio20.reconfigure(),
    );

    // let mut wave =
    //     SimpleFormSource::infinite_mono(I2S_SAMPLE_RATE, WaveForm::Triangle, 240.0).into_iter();

    let freq = 440f32;
    let mut sample_index = 0usize;
    let sr = I2S_SAMPLE_RATE as f32;
    let period = sr / freq;
    let amplitude = u32::MAX as f32 * 0.1;

    loop {
        let point = sample_index as f32 / period;
        let s = ((PI2 * point).sin() * amplitude) as i32 as u32;
        i2s = i2s.write(s, s);

        if sample_index % 4 == 0 {
            i2s = i2s.send();
        }

        sample_index = sample_index.wrapping_add(1);
    }
}
