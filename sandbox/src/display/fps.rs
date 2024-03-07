use crate::bsp::hal::fugit::MicrosDurationU64;
use crate::bsp::hal::fugit::{HertzU64, NanosDurationU64};
use core::fmt::Write;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::FONT_9X18_BOLD;
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::{BinaryColor, Rgb565, RgbColor};
use embedded_graphics::text::{Baseline, Text};

use defmt::{write, Format, Formatter};

use crate::bsp::hal::{
    clocks::ClocksManager,
    fugit::{Duration, Hertz},
    pac::{RESETS, TIMER},
    timer::Instant,
    Timer,
};

pub struct FPS {
    timer: Timer,
    last_time: MicrosDurationU64,
    avg_sum: HertzU64,
    avg_cycles: u64,
    avg_period: Option<HertzU64>,
}

impl FPS {
    pub fn new(timer: Timer, avg_period: Option<HertzU64>) -> Self {
        Self {
            timer,
            last_time: MicrosDurationU64::from_ticks(0),
            avg_sum: HertzU64::from_raw(0),
            avg_cycles: 0,
            avg_period,
        }
    }

    pub fn tick(&mut self) -> Option<u64> {
        let now = self.timer.get_counter().duration_since_epoch();
        let fps = now.checked_sub(self.last_time)?.into_rate();

        self.last_time = now;

        if let Some(avg_period) = self.avg_period {
            if fps >= avg_period {
                self.avg_sum = fps;
                self.avg_cycles = 1;
            } else {
                self.avg_sum += fps;
                self.avg_cycles += 1;
            }
        }

        Some(fps.to_Hz())
    }

    pub fn avg(&mut self) -> Option<u64> {
        let _val = self.tick()?;
        Some(self.avg_sum / HertzU64::Hz(self.avg_cycles))
    }

    // pub fn text<'a, C>(
    //     &'a mut self,
    //     text_style: MonoTextStyle<'a, C>,
    // ) -> Option<Text<MonoTextStyle<C>>> {
    //     let mut fps_str = heapless::String::<10>::new();
    //     let _ = write!(fps_str, "{}fps", self.val()?);

    //     Some(Text::with_baseline(
    //         &fps_str,
    //         Point::new(0, 0),
    //         text_style,
    //         Baseline::Top,
    //     ))
    // }
}
