#![no_main]
#![no_std]

//use panic_halt as _;
//use cortex_m::iprintln;
use stm32f4xx_hal as hal;
use crate::hal::{prelude::*, stm32};
use core::panic::PanicInfo;

use rtt_target::{rprintln, rtt_init_print};

// unused !!

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!(BlockIfFull);
    rprintln!("Hello, world!");

    loop {

    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}