#![no_main]
#![no_std]

mod common;
use common::main;

/* todo:
 * - board init: super::super::tests::runner(some_struct);
 */

#[bern_test::tests]
mod tests {
    #[tear_down]
    fn reset() {
        // add a short delay to flush serial
        // todo: add wait functionality
        let mut i = 0;
        while i < 1000 {
            i += 1;
        }
        cortex_m::peripheral::SCB::sys_reset();
    }

    #[test]
    fn first_test() {
        assert!(1 == 1, "wrong");
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