#![no_main]
#![no_std]

mod common;
use common::main;

fn recursion(i: u32) {
    recursion(i+1);
}

#[bern_test::tests]
mod tests {
    use crate::common::st_nucleo_f446::StNucleoF446;
    use stm32f4xx_hal::prelude::*;

    #[test_tear_down]
    fn reset() {
        // add a short delay to flush serial
        // todo: add wait functionality
        super::common::stupid_wait(1000);
        cortex_m::peripheral::SCB::sys_reset();
    }

    #[tear_down]
    fn stop() {
        cortex_m::asm::bkpt();
    }

    #[test]
    fn should_fail() {
        assert!(1 == 0, "wrong");
    }

    #[test]
    fn with_board(board: &mut StNucleoF446) {
        board.led.set_high().ok();
        board.shield.led_0.set_high().ok();
    }

    #[test]
    #[ignored]
    fn stack_overflow() {
        super::recursion(0);
    }

    #[test]
    #[should_panic]
    fn another_test() {
        assert!(1 == 0, "wrong");
    }

    #[test]
    #[ignored]
    fn a_third_test() {
        assert!(42 == 42, "wow");
    }
}