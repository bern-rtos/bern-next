//#![deny(unsafe_code)]
#![no_main]
#![no_std]

use bern_kernel as kernel;
use kernel::{
    task,
    scheduler::Scheduler,
    Syscall,
};

use panic_halt as _;

use cortex_m;
use cortex_m_rt::entry;
use st_nucleo_f446::StNucleoF446;
use stm32f4xx_hal::prelude::*;

#[entry]
fn main() -> ! {
    cortex_m::asm::bkpt();
    let board = StNucleoF446::new();

    Scheduler::init();
    /* idle task */
    task::spawn(move | | {
        loop {
            cortex_m::asm::nop();
        }
    },
                kernel::alloc_static_stack!(128)
    );

    /* task 1 */
    let mut led = board.shield.led_7;
    task::spawn(move || {
        loop {
            led.toggle().ok();
            kernel::Core::sleep(100);
        }
    },
                kernel::alloc_static_stack!(512)
    );

    /* task 2 */
    let mut another_led = board.shield.led_6;
    task::spawn(move || {
        /* spawn a new task while the system is running */
        task::spawn(move || {
            loop {
                kernel::Core::sleep(800);
            }
        },
                    kernel::alloc_static_stack!(256)
        );

        loop {
            another_led.set_high().ok();
            kernel::Core::sleep(50);
            another_led.set_low().ok();
            kernel::Core::sleep(400);
        }
    },
                kernel::alloc_static_stack!(1024)
    );


    let mut yet_another_led = board.shield.led_1;
    let mut a = 0;
    task::spawn(move || {
        loop {
            a += 1;
            yet_another_led.set_high().ok();
            kernel::Core::sleep(50);
            yet_another_led.set_low().ok();
            kernel::Core::sleep(950);
        }
    },
                kernel::alloc_static_stack!(512)
    );

    Scheduler::start();
}
