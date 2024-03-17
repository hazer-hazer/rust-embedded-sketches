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
    dsp::math::F32Ext,
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
const I2S_SAMPLE_RATE: u32 = 16000;
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

    let mut led = pins.led.into_push_pull_output();

    // let spi = Spi::<_, _, _, 8>::new(
    //     pac.SPI0,
    //     (pins.gpio3.into_function(), pins.gpio2.into_function()),
    // );

    // let mut cs = pins.gpio4.into_push_pull_output();
    // let dc = pins.gpio6.into_push_pull_output();
    // let rst = pins.gpio5.into_push_pull_output();

    // let spi = spi.init(
    //     &mut pac.RESETS,
    //     clocks.peripheral_clock.freq(),
    //     16_000_000.Hz(),
    //     bsp::hal::spi::FrameFormat::MotorolaSpi(embedded_hal::spi::MODE_0),
    // );

    // cs.set_low().unwrap();

    // let mut display =
    //     st7735_lcd::ST7735::new(spi, dc, rst, true, true, DISPLAY_WIDTH, DISPLAY_HEIGHT);

    // display.init(&mut delay).unwrap();

    // display
    //     .set_orientation(&st7735_lcd::Orientation::Landscape)
    //     .unwrap();

    // display.set_offset(1, 26);
    // display.clear(Rgb565::BLACK).unwrap();

    // let pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
    // let mut pwm = pwm_slices.pwm5.into_mode::<pwm::FreeRunning>();
    // pwm.default_config();
    // // pwm.set_ph_correct();

    // pwm.output_to(pins.gpio10);

    // let sample_rate = 44100;

    // let sys_clock = clocks.system_clock.freq().to_Hz();
    // // let top: u16 = (sys_clock / sample_rate / 2) as u16;
    // let top = sys_clock / sample_rate;
    // pwm.set_top(top as u16);
    // pwm.set_div_int(1);
    // pwm.enable();

    // let wave_freq = 440.0;

    // for form in WaveForm::VALUES.iter().cloned().cycle() {
    //     info!("Waveform: {}", form);
    //     let source = SimpleFormSource::infinite_mono(
    //         (DISPLAY_WIDTH as f32 * wave_freq) as u32,
    //         form,
    //         wave_freq,
    //     );

    //     display.clear(Rgb565::BLACK).unwrap();
    //     for (x, s) in source.enumerate().take(DISPLAY_WIDTH as usize) {
    //         defmt::assert!(s >= -1.0 && s <= 1.0);

    //         display
    //             .set_pixel(
    //                 x as u16,
    //                 s.remap_uint_range(0, DISPLAY_HEIGHT - 1) as u16,
    //                 Rgb565::WHITE.into_storage(),
    //             )
    //             .unwrap();
    //         // info!("x: {}, s: {}", x, s);
    //     }
    //     delay.delay_ms(5000);
    // }

    // let pio_test_prog = pio_asm!`(
    //     "
    //     .wrap_target
    //         set pins, 0b1 [2]
    //         set pins, 0b0 []
    //     .wrap
    // "
    // );

    // let pio_test_pin = pins.gpio21.into_function::<FunctionPio1>();
    // let clock_div_int = 1000;
    // debug!(
    //     "Run test square wave PIO program on pin {} with frequency of sys/{}",
    //     pio_test_pin.id().num,
    //     clock_div_int
    // );
    // let (mut pio1, sm0, _, _, _) = pac.PIO1.split(&mut pac.RESETS);
    // let (mut sm0, _, _) = PIOBuilder::from_program(pio1.install(&pio_test_prog.program).unwrap())
    //     .set_pins(21, 1)
    //     .clock_divisor_fixed_point(clock_div_int, 0)
    //     .build(sm0);
    // sm0.set_pindirs([(pio_test_pin.id().num, PinDir::Output)]);

    // sm0.start();

    // led.set_high().unwrap();
    // info!("Starting main loop");

    // let sine = SimpleFormSource::infinite_mono(I2S_SAMPLE_RATE, WaveForm::Sine, 440.0);
    // // let wave_len = sine.wave_len();
    // // let sine_samples = sine.take(wave_len).collect::<Vec<_>>();

    // // for s in sine_samples.iter().cycle() {
    // //     i2s = i2s.write_both(s * 0.2).maybe_send();
    // // }

    // for s in sine {
    //     i2s = i2s.write_both(s * 2.0).maybe_send();
    // }

    let mut dma = pac.DMA.split(&mut pac.RESETS);

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

    let din = pins.gpio18.into_function::<FunctionPio0>();
    let bclk = pins.gpio19.into_function::<FunctionPio0>();
    let lrck = pins.gpio20.into_function::<FunctionPio0>();

    let pio_clock_div = clocks.system_clock.freq().to_Hz() as f32 / (I2S_SAMPLE_RATE as f32 * 64.0);
    defmt::assert!(pio_clock_div.fract() == 0.0);
    let pio_clock_div = pio_clock_div as u16;

    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let i2s_out_master_prog =
        pio_proc::pio_file!("./src/rp2040_ext/i2s.pio", select_program("i2s_out_master"));
    let i2s_out_master_prog = pio.install(&i2s_out_master_prog.program).unwrap();
    let (mut sm0, _, mut tx) = PIOBuilder::from_program(i2s_out_master_prog)
        .out_pins(din.id().num, 1)
        .side_set_pin_base(bclk.id().num)
        .clock_divisor_fixed_point(pio_clock_div, 0)
        .out_shift_direction(hal::pio::ShiftDirection::Left)
        .autopull(true)
        .pull_threshold(32)
        .buffers(hal::pio::Buffers::OnlyTx)
        .build(sm0);

    sm0.set_pindirs([
        (din.id().num, PinDir::Output),
        (bclk.id().num, PinDir::Output),
        (lrck.id().num, PinDir::Output),
    ]);

    sm0.start();

    const BUFFER_SIZE: usize = 16;
    // Merged as mono -> stereo
    // let mut sample_queue = Queue::<f32, { BUFFER_SIZE * 4 + 1 }>::new();
    let buf1 = singleton!(: [u32; BUFFER_SIZE] = [0; BUFFER_SIZE]).unwrap();
    let buf2 = singleton!(: [u32; BUFFER_SIZE] = [0; BUFFER_SIZE]).unwrap();

    let mut dma_cfg = hal::dma::double_buffer::Config::new((dma.ch0, dma.ch1), buf1, tx);
    // dma_cfg.pace(Pace::PreferSink);
    let transfer = dma_cfg.start();
    let mut transfer = transfer.read_next(buf2);

    // let mut local_buf = singleton!(: [u32; BUFFER_SIZE] = [0; BUFFER_SIZE]).unwrap();

    // let mut dma_cfg = hal::dma::single_buffer::Config::new(dma.ch0, buf1, tx);
    // let mut transfer = dma_cfg.start();

    let mut wave =
        SimpleFormSource::infinite_mono(I2S_SAMPLE_RATE, WaveForm::Triangle, 240.0).into_iter();

    let mut buf_queue = Queue::<[u32; BUFFER_SIZE], 16>::new();

    // Prefill queue
    while !buf_queue.is_full() {
        let mut buf = [0; BUFFER_SIZE];
        let mut prev_channel = 0;
        for (i, e) in buf.iter_mut().enumerate() {
            if i % 2 == 0 {
                *e = (wave.next().unwrap() * 0.1 * i32::MAX as f32) as i32 as u32;
                prev_channel = *e;
            } else {
                *e = prev_channel;
            }
        }
        buf_queue.enqueue(buf).unwrap();
    }

    // let mut buf_ptr = 0usize;
    // let mut buffers = [[0; BUFFER_SIZE]; 100];

    // for buf in buffers.iter_mut() {
    //     let mut prev_channel = 0;
    //     for (i, e) in buf.iter_mut().enumerate() {
    //         if i % 2 == 0 {
    //             *e = (sine.next().unwrap() * 0.1 * i32::MAX as f32) as i32 as u32;
    //             prev_channel = *e;
    //         } else {
    //             *e = prev_channel;
    //         }
    //     }
    // }

    loop {
        // if transfer.is_done() {
        //     let (next_buf, next_transfer) = transfer.wait();
        //     // TODO: Set to empty or replay last if no buffer in queue
        //     *next_buf = buffers[buf_ptr % buffers.len()];

        //     // info!("Send next buffer from queue:\n{}", next_buf);
        //     transfer = next_transfer.read_next(next_buf);

        //     buf_ptr += 1;
        // }
        // if transfer.is_done() {
        //     let (ch0, buf, next_tx) = transfer.wait();

        //     *buf = buffers[buf_ptr % buffers.len()];
        //     buf_ptr += 1;

        //     let dma_cfg = hal::dma::single_buffer::Config::new(ch0, buf, next_tx);
        //     transfer = dma_cfg.start();
        // }
        if transfer.is_done() {
            let (next_buf, next_transfer) = transfer.wait();
            *next_buf = buf_queue.dequeue().unwrap();
            transfer = next_transfer.read_next(next_buf);
        }
        if !buf_queue.is_full() {
            // info!("Write next buffer to the queue");
            let mut buf = [0; BUFFER_SIZE];
            for e in buf.iter_mut() {
                *e = (wave.next().unwrap() * 0.1 * i32::MAX as f32) as i32 as u32;
            }
            buf_queue.enqueue(buf).unwrap();
        } else {
            // info!("Queue is full, failed to write next buffer");
        }

        // if transfer.is_done() {
        //     let (next_buf, next_transfer) = transfer.wait();

        //     transfer = next_transfer.read_next(local_buf);
        //     local_buf = next_buf;

        //     for s in local_buf.iter_mut() {
        //         *s = (sine.next().unwrap() * 0.1 * i32::MAX as f32) as i32 as u32;
        //         if transfer.is_done() {
        //             break;
        //         }
        //     }
        // }
    }
}
