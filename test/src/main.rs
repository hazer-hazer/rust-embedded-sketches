#![no_std]
#![no_main]

// pick a panicking behavior
// use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_semihosting::{debug, hprintln};

#[cortex_m_rt::entry]
fn main() -> ! {
    hprintln!("KEK");

    debug::exit(debug::EXIT_SUCCESS);

    loop {
        // your code goes here
    }
}
