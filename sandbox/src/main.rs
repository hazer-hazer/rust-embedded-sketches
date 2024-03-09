//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use defmt::*;
use embedded_hal::digital::v2::OutputPin;

extern crate sandbox;

use num::{pow::Pow, ToPrimitive};
use sandbox::{bsp, dsp::math::F32Ext, exit, heap::init_global_heap, Vec};

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

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
// use rp_pico as bsp;
// use waveshare_rp2040_zero as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::{
    entry,
    hal::{
        self,
        clocks::{init_clocks_and_plls, Clock},
        fugit::{HertzU64, RateExtU32},
        gpio::{bank0::Gpio0, FunctionUart, PullDown, PullNone, PullUp},
        pac,
        sio::Sio,
        spi::FrameFormat,
        uart::{self, DataBits, Disabled, StopBits, UartConfig, UartPeripheral, ValidUartPinout},
        watchdog::Watchdog,
        Spi, Timer,
    },
    pac::{uart0, UART0},
    XOSC_CRYSTAL_FREQ,
};

use core::fmt::Write;
use embedded_text::{style::TextBoxStyleBuilder, TextBox};

const DISPLAY_WIDTH: u32 = 160;
const DISPLAY_HEIGHT: u32 = 80;

// const DISPLAY_WIDTH: u32 = 240;
// const DISPLAY_HEIGHT: u32 = 240;

#[entry]
fn main() -> ! {
    info!("Program start");

    unsafe { init_global_heap() };

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let external_xtal_freq_hz = XOSC_CRYSTAL_FREQ;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led = pins.led.into_push_pull_output();

    // loop {
    //     info!("KEK");
    //     led.set_high().unwrap();
    //     delay.delay_ms(1000);

    //     info!("KEK");
    //     led.set_low().unwrap();
    //     delay.delay_ms(1000);
    // }

    // static SERIAL: StaticCell<
    //     bsp::hal::uart::UartPeripheral<
    //         bsp::hal::uart::Enabled,
    //         pac::UART0,
    //         (
    //             bsp::hal::gpio::Pin<bsp::hal::gpio::bank0::Gpio0, FunctionUart, PullNone>,
    //             bsp::hal::gpio::Pin<bsp::hal::gpio::bank0::Gpio1, FunctionUart, PullNone>,
    //         ),
    //     >,
    // > = StaticCell::new();

    // let mut uart = bsp::hal::uart::UartPeripheral::new(
    //     pac.UART0,
    //     (pins.gp0.into_function(), pins.gp1.into_function()),
    //     &mut pac.RESETS,
    // )
    // .enable(
    //     UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
    //     clocks.peripheral_clock.freq(),
    // )
    // .unwrap();

    // defmt_serial::defmt_serial(SERIAL.init(uart));

    let spi = Spi::<_, _, _, 8>::new(
        pac.SPI0,
        (pins.gpio3.into_function(), pins.gpio2.into_function()),
    );

    let mut cs = pins.gpio4.into_push_pull_output();
    let dc = pins.gpio6.into_push_pull_output();
    let rst = pins.gpio5.into_push_pull_output();

    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16_000_000.Hz(),
        bsp::hal::spi::FrameFormat::MotorolaSpi(embedded_hal::spi::MODE_0),
    );

    cs.set_low().unwrap();

    let mut display =
        st7735_lcd::ST7735::new(spi, dc, rst, true, true, DISPLAY_WIDTH, DISPLAY_HEIGHT);

    display.init(&mut delay).unwrap();

    display
        .set_orientation(&st7735_lcd::Orientation::Landscape)
        .unwrap();

    display.set_offset(1, 26);
    display.clear(Rgb565::BLACK).unwrap();

    // let di = display_interface_spi::SPIInterfaceNoCS::new(spi, dc);
    // let mut display = mipidsi::Builder::st7789(di)
    //     .with_display_size(DISPLAY_WIDTH as u16, DISPLAY_HEIGHT as u16)
    //     .with_orientation(mipidsi::Orientation::LandscapeInverted(true))
    //     .with_invert_colors(mipidsi::ColorInversion::Inverted)
    //     .init(&mut delay, Some(rst))
    //     .unwrap();

    let char_style = MonoTextStyleBuilder::new()
        .font(&FONT_7X13)
        .text_color(Rgb565::WHITE)
        .background_color(Rgb565::BLACK)
        .build();

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut fps = FPS::new(timer, Some(HertzU64::Hz(1)));

    const FPS_BOX: (usize, usize) = (7 * 6, 13);

    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(embedded_text::style::HeightMode::FitToText)
        .alignment(embedded_text::alignment::HorizontalAlignment::Center)
        .build();
    let textbox_bounds =
        Rectangle::new(Point::zero(), Size::new(FPS_BOX.0 as u32, FPS_BOX.1 as u32));

    let mut fmt_buf = FmtBuf::<64>::new();

    let sample_rate = 44000;
    let sine_wave = sine_wave(sample_rate, 440.0, 1.0);

    let resample_rate = DISPLAY_WIDTH;
    let mut resampler = ResamplerBuilder::new(sample_rate, resample_rate)
        .no_lpf()
        .build();
    let resampled_sine_wave = resampler.resample(&sine_wave);
    defmt::assert_eq!(resampled_sine_wave.len(), DISPLAY_WIDTH as usize);

    info!("Resampled successfully");

    led.set_high().unwrap();

    const DISPLAY_BOX: Size = Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT);

    info!("Starting main loop");
    loop {
        {
            let mut data = [Rgb565::default(); FPS_BOX.0 * FPS_BOX.1];
            let mut fbuf = FrameBuf::new(&mut data, FPS_BOX.0, FPS_BOX.1);

            fmt_buf.reset();

            let fps_avg = fps.tick().unwrap();

            core::write!(fmt_buf, "{}fps", fps_avg).unwrap();

            TextBox::with_textbox_style(
                fmt_buf.as_str(),
                textbox_bounds,
                char_style,
                textbox_style,
            )
            .draw(&mut fbuf)
            .unwrap();

            display.fill_contiguous(&textbox_bounds, data).unwrap();
        }

        // {
        //     let mut data =
        //         [Rgb565::default(); DISPLAY_BOX.width as usize * DISPLAY_BOX.height as usize];
        //     let mut fbuf = FrameBuf::new(
        //         &mut data,
        //         DISPLAY_BOX.width as usize,
        //         DISPLAY_BOX.height as usize,
        //     );
        //     for (x, y) in resampled_sine_wave.iter().copied().enumerate() {
        //         fbuf.set_color_at(
        //             Point::new(
        //                 x as i32,
        //                 y.remap_to_range((0, DISPLAY_HEIGHT as u16)) as i32,
        //             ),
        //             Rgb565::WHITE,
        //         );
        //     }

        //     display
        //         .fill_contiguous(&Rectangle::new(Point::zero(), DISPLAY_BOX), data)
        //         .unwrap();
        // }

        for (x, y) in resampled_sine_wave.iter().copied().enumerate() {
            display
                .set_pixel(
                    x as u16,
                    y.remap_to_int_range(0, DISPLAY_HEIGHT) as u16,
                    Rgb565::WHITE.into_storage(),
                )
                .unwrap();
        }
    }
}
