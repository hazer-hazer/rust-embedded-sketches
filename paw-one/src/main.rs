#![no_std]
#![no_main]

extern crate paw_one;

use defmt::*;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Duration, Instant};
use embedded_sdmmc::ShortFileName;
use embedded_text::{
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
    TextBox,
};
use paw::{
    audio::{osc::simple_form::SimpleFormSource, source::AudioSourceIter},
    dsp::math::PI2,
};
use paw_one::{sd::FileReader, wav::WavHeader};

use {defmt_rtt as _, panic_probe as _};

use core::{cell::RefCell, fmt::Write};
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    dac::{self, Dac, DacCh1, DacDma2},
    dma::{self, NoDma},
    gpio::Output,
    i2c::I2c,
    peripherals,
    rcc::mux::ClockMux,
    spi,
    time::Hertz,
};
use embedded_graphics::{
    mock_display::ColorMapping,
    mono_font::{ascii::FONT_6X10, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    primitives::Rectangle,
    text::Text,
};

mod display;

const SAMPLE_RATE: u32 = 48_000;
const AUDIO_BUFFER_SIZE: usize = 512;

fn dir_tree<
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
    const MAX_DIRS: usize,
    const MAX_FILES: usize,
    const MAX_VOLUMES: usize,
>(
    mut dir: embedded_sdmmc::Directory<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    indent: usize,
) {
    let mut children = heapless::Vec::<ShortFileName, 16>::new();
    dir.iterate_dir(|e| {
        info!(
            "- {} {:a} {} {}B",
            if e.attributes.is_directory() {
                "<DIR>"
            } else if e.attributes.is_system() {
                "<SYS>"
            } else if e.attributes.is_volume() {
                "<VOL>"
            } else if e.attributes.is_lfn() {
                "<LFN>"
            } else if e.attributes.is_hidden() {
                "<HID>"
            } else {
                ""
            },
            e.name,
            e.mtime,
            e.size
        );

        if e.attributes.is_directory()
            && e.name != embedded_sdmmc::ShortFileName::parent_dir()
            && e.name != embedded_sdmmc::ShortFileName::this_dir()
        {
            children.push(e.name.clone()).unwrap();
        }
    })
    .unwrap();
    for child_name in children {
        let child_dir = dir.open_dir(&child_name).unwrap();
        dir_tree(child_dir, indent + 2);
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Program entered");

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

    let mut p = embassy_stm32::init(config);

    let amp = 0.5;

    let mut sd_spi_cfg = spi::Config::default();
    sd_spi_cfg.frequency = Hertz::mhz(24);
    let sd_spi = spi::Spi::new(p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA2, p.DMA2_CH2, sd_spi_cfg);
    let sd_spi_bus: embassy_sync::blocking_mutex::Mutex<NoopRawMutex, _> =
        embassy_sync::blocking_mutex::Mutex::new(RefCell::new(sd_spi));
    let sd_spi_device = embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice::new(
        &sd_spi_bus,
        embedded_sdmmc::sdcard::DummyCsPin,
    );
    let sd_card = embedded_sdmmc::sdcard::SdCard::new(
        sd_spi_device,
        Output::new(
            p.PA4,
            embassy_stm32::gpio::Level::High,
            embassy_stm32::gpio::Speed::High,
        ),
        embassy_time::Delay,
    );

    struct TimeSourceStub;
    impl embedded_sdmmc::TimeSource for TimeSourceStub {
        fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
            embedded_sdmmc::Timestamp::from_calendar(2000, 7, 16, 0, 0, 0).unwrap()
        }
    }

    let time_source = TimeSourceStub;

    let mut volume_manager = embedded_sdmmc::VolumeManager::new(sd_card, time_source);

    let mut volume = volume_manager
        .open_volume(embedded_sdmmc::VolumeIdx(0))
        .unwrap();
    let mut root_dir = volume.open_root_dir().unwrap();
    // dir_tree(root_dir, 0);

    let wav_file_name = "NIRVANA.WAV";
    let file = root_dir
        .open_file_in_dir(wav_file_name, embedded_sdmmc::Mode::ReadOnly)
        .unwrap();

    let mut file_reader: FileReader<1024, _, _, 4, 4, 1> = FileReader::new(file);

    let wav_header = WavHeader::by_file_reader(&mut file_reader).unwrap();

    info!("WAV Header: {:a}", wav_header);

    let dma_buf =
        cortex_m::singleton!(: [u16; AUDIO_BUFFER_SIZE] = [0; AUDIO_BUFFER_SIZE]).unwrap();

    let (sai_block_a, _) = embassy_stm32::sai::split_subblocks(p.SAI1);
    let sai_cfg =
        paw_one::audio::sai::Sai::cfg_i2s(wav_header.sample_rate, wav_header.bits_per_sample as u8);

    let mut sai = embassy_stm32::sai::Sai::new_asynchronous_with_mclk(
        sai_block_a,
        p.PA8,
        p.PA10,
        p.PA9,
        p.PA3,
        p.DMA1_CH1,
        dma_buf,
        sai_cfg,
    );

    let mut sent_count = 0;
    let mut underrun_count = 0;
    // Micros last
    let mut current_period = 0;

    info!(
        "SYS CLOCK FREQUENCY: {}",
        embassy_stm32::rcc::frequency::<peripherals::SYSCFG>()
    );

    // let mut sound = SimpleFormSource::infinite_stereo(
    //     SAMPLE_RATE,
    //     paw::audio::osc::simple_form::WaveForm::Sine,
    //     480.0,
    // );

    info!("Starting main loop");
    loop {
        // for freq in notes {
        //     // sound.set_freq(freq);
        //     let period = SAMPLE_RATE as f32 / freq;
        //     let occupied_len = period as usize * 2;
        //     // Buffer size of 512 gives us clear minimum of 93.75Hz, but 256 -> 187.5Hz (not enough)
        //     // But we always have two channels, thus the real count of samples is divided by 2.
        //     let mut sine = [0; BUFFER_SIZE];
        //     let mut last_val = 0;
        //     defmt::assert!(sine.len() >= occupied_len);
        //     for (i, s) in sine[0..occupied_len].iter_mut().enumerate() {
        //         use micromath::F32Ext;
        //         if i % 2 == 0 {
        //             *s = ((PI2 * i as f32 / period as f32).sin() * i32::MAX as f32 * amp) as i32
        //                 as u32;
        //             last_val = *s;
        //         } else {
        //             *s = last_val;
        //         }
        //     }
        //     // defmt::assert!(sine[0] == 0 && sine[occupied_len - 1] == 0);
        //     for _ in 0..repeat {
        //         sai.write(&sine[0..occupied_len]).await.unwrap();
        //     }
        // }

        // let mut buf = [0; BUFFER_SIZE];
        // let period = SAMPLE_RATE as f32 / FREQ;
        // let occupied_len = period as usize * 2;
        // // info!(
        // //     "Read {}B buffer in {:tus}:",
        // //     buf.len(),
        // //     bench_start.elapsed().as_micros()
        // // );

        // let mut last_val = 0;
        // for (i, s) in buf[0..occupied_len].iter_mut().enumerate() {
        //     use micromath::F32Ext;
        //     if i % 2 == 0 {
        //         *s = ((PI2 * i as f32 / period as f32).sin() * i16::MAX as f32 * amp) as i16 as u16;
        //         last_val = *s;
        //     } else {
        //         *s = last_val;
        //     }
        // }

        let bench_start = Instant::now();

        // let buf = file_reader.next_buf().unwrap();

        let mut buf = [0; AUDIO_BUFFER_SIZE];
        for s in buf.iter_mut() {
            *s = file_reader.next_be().unwrap();
            // *s = (sound.next_sample() * i16::MAX as f32) as i16 as u16;
        }

        current_period += bench_start.elapsed().as_micros();

        sai.start(); // NOTE: `start` JUST BEFORE FIRST `write`!!!

        match sai.write(&buf).await {
            Ok(_) => {}
            Err(_) => {
                underrun_count += 1;
                sai.flush();
            }
        }

        sent_count += 1;

        if current_period >= 1_000_000 {
            let secs = current_period as f32 / 1_000_000.0;
            println!(
                "Sent {}/{} buffers ({}B) with underrun in {}s",
                underrun_count,
                sent_count,
                buf.len(),
                secs
            );
            println!(
                "Speed: {}KiB/s",
                (sent_count * core::mem::size_of_val(&buf)) as f32 / 1_024.0 / secs
            );
            sent_count = 0;
            underrun_count = 0;
            current_period = 0;
        }
    }
}
