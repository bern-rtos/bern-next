//#![deny(unsafe_code)]
#![no_main]
#![no_std]

use bern_kernel as kernel;
use kernel::{
    task::Task,
    task::Priority,
    scheduler,
    sync::mutex::Mutex,
};

use panic_halt as _;

use cortex_m;
use cortex_m_rt::entry;
use st_nucleo_f446::StNucleoF446;
use stm32f4xx_hal::prelude::*;

//#[link_section = ".shared"]
static MUTEX: Mutex<u32> = Mutex::new(42);

#[entry]
fn main() -> ! {
    cortex_m::asm::bkpt();
    let board = StNucleoF446::new();

    scheduler::init();

    /* idle task */
    Task::new()
        .idle_task()
        .static_stack(kernel::alloc_static_stack!(128))
        .spawn(move || {
            loop {
                cortex_m::asm::nop();
            }
        });

    /* task 1 */
    let mut led = board.shield.led_7;
    Task::new()
        .priority(Priority(1))
        .static_stack(kernel::alloc_static_stack!(512))
        .spawn(move || {
            loop {
                led.toggle().ok();
                kernel::sleep(100);
                {
                    match MUTEX.try_lock() {
                        Ok(mut value) => *value = 54,
                        Err(_) => (),
                    }
                }
            }
        });

    /* task 2 */
    let mut another_led = board.shield.led_6;
    Task::new()
        .priority(Priority(2))
        .static_stack(kernel::alloc_static_stack!(1024))
        .spawn(move || {
            /* spawn a new task while the system is running */
            Task::new()
                .static_stack(kernel::alloc_static_stack!(512))
                .spawn(move || {
                    loop {
                        kernel::sleep(800);
                    }
                });

            loop {
                another_led.set_high().ok();
                kernel::sleep(50);
                another_led.set_low().ok();
                kernel::sleep(400);

                match MUTEX.try_lock() {
                    Ok(mut value) => *value = 36,
                    Err(_) => (),
                }
            }
        });


    let mut yet_another_led = board.shield.led_1;
    let mut a = 10;
    Task::new()
        .priority(Priority(3))
        .static_stack(kernel::alloc_static_stack!(512))
        .spawn(move || {
            loop {
                a += 1;
                yet_another_led.set_high().ok();
                kernel::sleep(50);
                yet_another_led.set_low().ok();
                kernel::sleep(950);
                if a >= 60 {
                    kernel::task_exit();
                }
            }
        });

    /* blocking task */
    Task::new()
        .priority(Priority(2))
        .static_stack(kernel::alloc_static_stack!(128))
        .spawn(move || {
            loop {
                cortex_m::asm::nop();
            }
        });

    scheduler::start();
}
