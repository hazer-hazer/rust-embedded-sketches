#![no_std]
#![no_main]

extern crate alloc;
extern crate paw_one;

use defmt::*;
use display_interface_spi::SPIInterface;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics_framebuf::FrameBuf;
use embedded_sdmmc::ShortFileName;
use embedded_text::{
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
    TextBox,
};
use paw::{
    audio::{
        osc::simple_form::SimpleFormSource,
        source::{AudioSourceIter, Channel},
    },
    dsp::math::PI2,
    ui::audio::wave_window::{WaveWindow, Window},
};
use paw_one::{heap::init_global_heap, sd::FileReader, wav::WavHeader};

use crate::display::fps::FPS;

use {defmt_rtt as _, panic_probe as _};

use core::{cell::RefCell, fmt::Write};
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    dac::{self, Dac, DacCh1, DacDma2},
    dma::{self, NoDma},
    gpio::Output,
    i2c::{self, I2c},
    peripherals,
    rcc::mux::ClockMux,
    spi,
    time::Hertz,
};
use embedded_graphics::{
    mock_display::ColorMapping,
    mono_font::{self, ascii::FONT_6X10, MonoFont, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, StyledDrawable},
    text::{renderer::CharacterStyle, Alignment, Text},
};
use ssd1306::prelude::*;

mod display;

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

const SAMPLE_RATE: u32 = 96_000;
const BIT_DEPTH: u8 = 32;
const CHANNELS_COUNT: usize = 2;
const AUDIO_BUFFER_SIZE: usize = 256;
const SINGLE_CHANNEL_BUFFER_SIZE: usize = AUDIO_BUFFER_SIZE / CHANNELS_COUNT;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Program entered");

    // unsafe { init_global_heap() };
    // {
    //     let mut vec = alloc::vec![1, 2, 3, 4, 5];
    //     vec.push(1);
    //     vec.pop();
    //     info!("HEAP Check with vector ran successfully");
    // }

    let mut config = embassy_stm32::Config::default();

    {
        // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=59adafc115d9d0e4238be823d8b066c8
        use embassy_stm32::rcc::*;

        config.rcc.pll = Some(embassy_stm32::rcc::Pll {
            // 16MHz HSI
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV5,
            mul: PllMul::MUL92,
            divp: Some(PllPDiv::DIV2),
            divq: Some(PllQDiv::DIV2),
            divr: Some(PllRDiv::DIV2),
        });
        config.rcc.sys = Sysclk::PLL1_R;
        // config.rcc.boost = true;
    }

    let p = embassy_stm32::init(config);

    info!(
        "SYS CLOCK FREQUENCY: {}",
        embassy_stm32::rcc::frequency::<peripherals::SYSCFG>()
    );

    // let mut w25q = {
    //     let mut w25q_spi_cfg = spi::Config::default();
    //     w25q_spi_cfg.frequency = Hertz::mhz(50);
    //     let w25q_spi = spi::Spi::new(
    //         p.SPI2,
    //         p.PB13,
    //         p.PB15,
    //         p.PB14,
    //         p.DMA1_CH3,
    //         p.DMA1_CH4,
    //         w25q_spi_cfg,
    //     );

    //     info!(
    //         "Configured SPI2 to {}Hz",
    //         w25q_spi.get_current_config().frequency
    //     );

    //     let mut w25q = w25q::series25::Flash::init(
    //         w25q_spi,
    //         Output::new(
    //             p.PB12,
    //             embassy_stm32::gpio::Level::Low,
    //             embassy_stm32::gpio::Speed::High,
    //         ),
    //     )
    //     .unwrap();

    //     info!(
    //         "W25Q Capacity: {}KB",
    //         w25q.get_device_info().unwrap().capacity_kb
    //     );

    //     w25q
    // };

    // ST7789 without CS
    // let mut display = {
    //     let mut display_spi_cfg = spi::Config::default();
    //     display_spi_cfg.frequency = Hertz::mhz(50);
    //     display_spi_cfg.mode = spi::MODE_2;
    //     let display_spi = spi::Spi::new_txonly(
    //         p.SPI1,
    //         p.PA5,
    //         p.PA7,
    //         p.DMA2_CH1,
    //         p.DMA2_CH2,
    //         display_spi_cfg,
    //     );

    //     info!(
    //         "Configured SPI1 to {}MHz",
    //         display_spi.get_current_config().frequency.0 as f32 / 1e6
    //     );

    //     let display_interface = display_interface_spi::SPIInterfaceNoCS::new(
    //         display_spi,
    //         Output::new(
    //             p.PA6,
    //             embassy_stm32::gpio::Level::Low,
    //             embassy_stm32::gpio::Speed::VeryHigh,
    //         ),
    //     );

    //     let mut display = mipidsi::Builder::st7789(display_interface)
    //         .with_display_size(240, 240)
    //         .with_invert_colors(mipidsi::ColorInversion::Inverted)
    //         .init(
    //             &mut embassy_time::Delay,
    //             Some(Output::new(
    //                 p.PA4,
    //                 embassy_stm32::gpio::Level::High,
    //                 embassy_stm32::gpio::Speed::Low,
    //             )),
    //         )
    //         .unwrap();

    //     display.clear(Rgb565::RED).unwrap();
    //     display
    // };

    let mut display = {
        let display_i2c_cfg = i2c::Config::default();
        let display_i2c = i2c::I2c::new(
            p.I2C1,
            p.PA15,
            p.PB7,
            Irqs,
            NoDma,
            NoDma,
            Hertz::mhz(1),
            display_i2c_cfg,
        );

        let di = ssd1306::I2CDisplayInterface::new(display_i2c);
        let mut display = ssd1306::Ssd1306::new(
            di,
            ssd1306::size::DisplaySize128x32,
            ssd1306::rotation::DisplayRotation::Rotate0,
        )
        .into_buffered_graphics_mode();
        display.init().unwrap();

        info!("Initialized SSD1306 display");

        display
    };

    display
        .bounding_box()
        .draw_styled(
            &PrimitiveStyle::with_stroke(BinaryColor::On, 1),
            &mut display,
        )
        .unwrap();

    display.flush().unwrap();

    let mut sai = {
        let sai_dma_buf =
            cortex_m::singleton!(: [u32; AUDIO_BUFFER_SIZE] = [0; AUDIO_BUFFER_SIZE]).unwrap();

        let (sai_block_a, _) = embassy_stm32::sai::split_subblocks(p.SAI1);
        let sai_cfg = paw_one::audio::sai::Sai::cfg_i2s(SAMPLE_RATE, BIT_DEPTH);

        let sai = embassy_stm32::sai::Sai::new_asynchronous_with_mclk(
            sai_block_a,
            p.PA8,
            p.PA10,
            p.PA9,
            p.PA3,
            p.DMA1_CH1,
            sai_dma_buf,
            sai_cfg,
        );

        sai
    };

    let mut sound = SimpleFormSource::infinite_stereo(
        SAMPLE_RATE,
        paw::audio::osc::simple_form::WaveForm::Sine,
        480.0,
    );

    display
        .bounding_box()
        .draw_styled(
            &PrimitiveStyle::with_stroke(BinaryColor::On, 1),
            &mut display,
        )
        .unwrap();

    display.flush().unwrap();

    // let fbuf_data =
    //     [Rgb565::BLACK; WAVE_WINDOW_SIZE.width as usize * WAVE_WINDOW_SIZE.height as usize];
    // let mut fbuf = FrameBuf::new(
    //     fbuf_data,
    //     WAVE_WINDOW_SIZE.width as usize,
    //     WAVE_WINDOW_SIZE.height as usize,
    // );
    // let fbuf_area = Rectangle::new(Point::new(0, 0), fbuf.size());

    const FPS_FONT: MonoFont = mono_font::ascii::FONT_4X6;

    let mut fps = FPS::new();

    let fps_text_style = MonoTextStyleBuilder::new()
        .background_color(BinaryColor::Off)
        .text_color(BinaryColor::On)
        .font(&FPS_FONT)
        .build();
    let fps_bounds_size = Size::new(
        FPS_FONT.character_size.width * 10 as u32,
        FPS_FONT.character_size.height,
    );
    let fps_bounds = Rectangle::new(
        Point::new(
            0,
            (display.bounding_box().size.height - fps_bounds_size.height) as i32,
        ),
        fps_bounds_size,
    );

    let mut sai_sent_count = 0;
    let mut underrun_count = 0;
    let mut last_bench = Instant::now();
    // Micros spent sending SAI data
    let mut sai_time = 0;

    let mut fps_time = Instant::now();

    const WAVE_WINDOW_POS: Point = Point::new(0, 0);
    const WAVE_WINDOW_SIZE: Size = Size::new(128, 32);
    const WAVE_WINDOW_AREA: Rectangle = Rectangle::new(WAVE_WINDOW_POS, WAVE_WINDOW_SIZE);
    let mut wave_window = WaveWindow::<128, _>::new(
        WAVE_WINDOW_POS,
        WAVE_WINDOW_SIZE,
        BinaryColor::On,
        Channel::stereo_first(),
        Window::Half,
    );

    sai.start(); // NOTE: `start` JUST BEFORE FIRST `write`!!!

    info!("Starting main loop");
    loop {
        let bench_start = Instant::now();

        let mut buf = [0; AUDIO_BUFFER_SIZE];
        for s in buf.iter_mut() {
            *s = (sound.next_sample() * i32::MAX as f32) as i32 as u32;
        }

        match sai.write(&buf).await {
            Ok(_) => {}
            Err(_) => {
                underrun_count += 1;
                sai.flush();
            }
        }

        sai_time += bench_start.elapsed().as_micros();

        sai_sent_count += 1;

        // if wave_window_captured >= wave_window_data.len() {
        //     display
        //         .fill_solid(&WAVE_WINDOW_AREA, BinaryColor::Off)
        //         .unwrap();
        //     WaveWindow::new(
        //         WAVE_WINDOW_AREA.top_left,
        //         WAVE_WINDOW_AREA.size,
        //         &wave_window_data,
        //         0,
        //         BinaryColor::On,
        //     )
        //     .draw(&mut display)
        //     .unwrap();

        //     display.flush().unwrap();

        //     wave_window_captured = 0;
        // } else {
        //     for i in 0..WAVE_WINDOW_CAPTURE_VALUES {
        //         wave_window_data[wave_window_captured] =
        //             buf[buf.len() - buf.len() / (i * CHANNELS_COUNT + 1)];
        //         wave_window_captured += 1;
        //     }
        // }

        wave_window.load(&buf);
        if wave_window.full() {
            display
                .fill_solid(&WAVE_WINDOW_AREA, BinaryColor::Off)
                .unwrap();
            wave_window.draw(&mut display).unwrap();
            display.flush().unwrap();
        }

        if last_bench.elapsed().as_micros() >= 1_000_000 {
            use micromath::F32Ext;

            let mut fps_text = heapless::String::<100>::new();
            core::write!(fps_text, "{}FPS", fps.value().round() as u32).unwrap();
            TextBox::with_alignment(
                &fps_text,
                fps_bounds,
                fps_text_style,
                HorizontalAlignment::Left,
            )
            .draw(&mut display)
            .unwrap();

            display.flush().unwrap();

            let sai_time_secs = sai_time as f32 / 1e6;

            println!(
                "Sent {}/{} buffers ({}B) with underrun in {}s",
                underrun_count,
                sai_sent_count,
                buf.len(),
                sai_time_secs,
            );
            println!(
                "SAI speed: {}KiB/s",
                (sai_sent_count as usize * core::mem::size_of_val(&buf)) as f32
                    / sai_time_secs
                    / 1_024.0
            );
            sai_sent_count = 0;
            underrun_count = 0;
            sai_time = 0;
            last_bench = Instant::now();
        }
    }
}
