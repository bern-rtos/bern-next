#![no_main]
#![no_std]

mod common;
use common::main as _;

#[bern_test::tests]
mod tests {
    use crate::common::st_nucleo_f446::StNucleoF446;
    use stm32f4xx_hal::prelude::*;
    use bern_kernel::scheduler::Scheduler;
    use bern_kernel::{alloc_static_stack, task::{Task}};

    #[test_set_up]
    fn init_scheduler() {
        Scheduler::init();
        /* idle task */
        Task::spawn(move | | {
            loop {
                cortex_m::asm::nop();
            }
        },
            alloc_static_stack!(128)
        );
    }

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
    fn first_task(board: &mut StNucleoF446) {
        let mut led = board.led.take().unwrap();

        Task::spawn(move | | {
            led.toggle().ok();
            Scheduler::sleep(100);

            assert_eq!(0,1);
        },
            alloc_static_stack!(2048) // need enough memory for panic handler...
        );
        Scheduler::start();
        loop {
            // todo: implement join() to wait for a thread to finish
        }
    }
}