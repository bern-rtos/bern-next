//#![deny(unsafe_code)]
#![no_main]
#![no_std]

use bern_kernel::{
    task::Task,
    scheduler::Scheduler,
    syscall,
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
    Task::spawn(move | | {
        loop {
            cortex_m::asm::nop();
        }
    },
                bern_kernel::alloc_static_stack!(128)
    );

    /* task 1 */
    let mut led = board.shield.led_7;
    Task::spawn(move || {
        loop {
            led.toggle().ok();
            Scheduler::delay(100);
        }
    },
                bern_kernel::alloc_static_stack!(512)
    );

    /* task 2 */
    let mut another_led = board.shield.led_6;
    Task::spawn(move || {
        /* spawn a new task while the system is running */
        Task::spawn(move || {
            loop {
                syscall::delay(800);
            }
        },
            bern_kernel::alloc_static_stack!(256)
        );

        loop {
            another_led.set_high().ok();
            syscall::delay(50);
            another_led.set_low().ok();
            syscall::delay(400);
        }
    },
                bern_kernel::alloc_static_stack!(512)
    );

    /*
    let mut another_led = board.shield.led_6;
    let mut a = 0;
    syscalls::spawn(move || {
        loop {
            a += 1;
            another_led.set_high().ok();
            Scheduler::delay(50);
            another_led.set_low().ok();
            Scheduler::delay(400);
        }
    });*/

    Scheduler::start();
    loop {
        panic!("We should have never arrived here!");
    }
}
