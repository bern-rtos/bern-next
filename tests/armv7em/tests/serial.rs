#![no_main]
#![no_std]

mod common;
use common::main;


fn recursion(i: u32) {
    recursion(i+1);
}

/* todo:
 * - board init: super::super::tests::runner(some_struct);
 */

#[bern_test::tests]
mod tests {
    use crate::common::st_nucleo_f446::StNucleoF446;
    use stm32f4xx_hal::prelude::*;

    #[tear_down]
    fn reset() {
        // add a short delay to flush serial
        // todo: add wait functionality
        super::common::stupid_wait(1000);
        cortex_m::peripheral::SCB::sys_reset();
    }

    #[test]
    fn should_fail(board: &mut StNucleoF446) {
        assert!(1 == 0, "wrong");
    }

    #[test]
    fn with_board(board: &mut StNucleoF446) {
        board.led.set_high().ok();
        board.shield.led_0.set_high().ok();
    }

    #[test]
    #[ignored]
    fn stack_overflow(board: &mut StNucleoF446) {
        super::recursion(0);
    }

    #[test]
    #[should_panic]
    fn another_test() {
        assert!(1 == 0, "wrong");
    }

    #[test]
    #[ignored]
    fn a_third_test(board: &mut StNucleoF446) {
        assert!(42 == 42, "wow");
    }
}