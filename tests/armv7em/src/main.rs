#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(my_runner)]
#![reexport_test_harness_main = "test_main"]

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

    #[cfg(test)]
    test_main();
    loop {

    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}

fn my_runner(tests: &[&i32]) {
    //let mut cortex_peripherals = cortex_m::peripheral::Peripherals::take().expect("cannot take cortex peripherals");
    //let stim = &mut cortex_peripherals.ITM.stim[1];

    for t in tests {
        if **t == 0 {
            rprintln!("PASSED");
        } else {
            rprintln!("FAILED");
        }
    }
}
