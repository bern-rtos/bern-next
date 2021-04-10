#![no_main]
#![no_std]
#![feature(test)]
#![feature(custom_test_frameworks)]
#![test_runner(my_runner)]

//use cortex_m::iprintln;
use core::panic::PanicInfo;
use stm32f4xx_hal as hal;
use crate::hal::{prelude::*, stm32};

use rtt_target::{rprintln, rtt_init_print};

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!(BlockIfFull);
    rprintln!("Running test...");

    //test_main();

    loop {

    }
}

fn my_runner(tests: &[&i32]) {
    rprintln!("test runner...");
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

#[test_case]
const WILL_PASS: i32 = 0;

#[test_case]
const WILL_FAIL: i32 = 4;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}


/*
#[defmt_test::tests]
mod tests {

    #[test]
    fn assert_true() {
        assert!(true)
    }

    #[test]
    fn assert_eq() {
        assert_eq!(24, 42, "TODO: write actual tests")
    }
}*/

