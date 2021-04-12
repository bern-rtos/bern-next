#![no_main]
#![no_std]

mod common;

#[bern_test::tests]
mod tests {
    #[tear_down]
    fn reset() {
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