#![no_std]
#![no_main]

#[macro_use]
extern crate micromath;

#[macro_use]
extern crate alloc;

pub mod audio;
pub mod display;
pub mod fmt;
// pub use waveshare_rp2040_zero as bsp;
pub use rp_pico as bsp;
pub mod dsp;
pub mod heap;
pub use alloc::vec::Vec;

use defmt_rtt as _;

use cortex_m_semihosting::debug;
// use panic_halt as _;
// use panic_semihosting as _;
use panic_probe as _;

#[inline(never)]
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

pub fn exit() -> ! {
    loop {
        debug::exit(debug::EXIT_SUCCESS);
    }
}

#[cortex_m_rt::exception]
unsafe fn HardFault(_frame: &cortex_m_rt::ExceptionFrame) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE);
    }
}
