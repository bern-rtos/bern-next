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
        super::common::stupid_wait(1000);
        cortex_m::peripheral::SCB::sys_reset();
    }

    #[test]
    fn first_test() {
        assert!(1 == 1, "wrong");
    }
}