#![deny(unsafe_code)]
#![no_main]
#![no_std]

use bern_kernel as kernel;
use kernel::{
    task::Task,
    task::Priority,
    sched,
    sync::Mutex,
};

use panic_halt as _;

use cortex_m;
use cortex_m_rt::entry;
use st_nucleo_f446::StNucleoF446;
use stm32f4xx_hal::prelude::*;
use bern_kernel::sync::Semaphore;

#[link_section = ".shared"]
static MUTEX: Mutex<u32> = Mutex::new(42);
#[link_section = ".shared"]
static SEMAPHORE: Semaphore = Semaphore::new(4);

#[entry]
fn main() -> ! {
    cortex_m::asm::bkpt();
    let board = StNucleoF446::new();

    sched::init();
    MUTEX.register().ok();
    SEMAPHORE.register().ok();

    /* idle task */
    Task::new()
        .idle_task()
        .static_stack(kernel::alloc_static_stack!(256))
        .spawn(move || {
            loop {
                cortex_m::asm::nop();
            }
        });

    // /* task 1 */
    // let mut led = board.shield.led_7;
    // Task::new()
    //     .priority(Priority(1))
    //     .static_stack(kernel::alloc_static_stack!(512))
    //     .spawn(move || {
    //         loop {
    //             {
    //                 match MUTEX.lock(1000) {
    //                     Ok(mut value) => *value = 54,
    //                     Err(_) => (),
    //                 }
    //             }
    //             led.toggle().ok();
    //             kernel::sleep(100);
    //         }
    //     });
    //
    // /* task 2 */
    // let mut another_led = board.shield.led_6;
    // Task::new()
    //     .priority(Priority(3))
    //     .static_stack(kernel::alloc_static_stack!(1024))
    //     .spawn(move || {
    //         /* spawn a new task while the system is running */
    //         Task::new()
    //             .static_stack(kernel::alloc_static_stack!(512))
    //             .spawn(move || {
    //                 loop {
    //                     kernel::sleep(800);
    //                 }
    //             });
    //
    //         loop {
    //             another_led.set_high().ok();
    //             match MUTEX.try_lock() {
    //                 Ok(_) => kernel::sleep(500),
    //                 Err(_) => (),
    //             }
    //             another_led.set_low().ok();
    //             kernel::sleep(1000);
    //         }
    //     });
    //
    //
    // let mut yet_another_led = board.shield.led_1;
    // let mut a = 10;
    // Task::new()
    //     .priority(Priority(3))
    //     .static_stack(kernel::alloc_static_stack!(512))
    //     .spawn(move || {
    //         loop {
    //             a += 1;
    //             yet_another_led.set_high().ok();
    //             kernel::sleep(50);
    //             yet_another_led.set_low().ok();
    //             kernel::sleep(150);
    //
    //             if a >= 60 {
    //                 let perm0 = SEMAPHORE.acquire(100);
    //                 let perm1 = SEMAPHORE.acquire(100);
    //                 let perm2 = SEMAPHORE.acquire(100);
    //                 let perm3 = SEMAPHORE.acquire(100);
    //                 let perm4 = SEMAPHORE.acquire(100);
    //                 core::mem::drop(perm0.ok().unwrap());
    //                 core::mem::drop(perm1.ok().unwrap());
    //                 core::mem::drop(perm2.ok().unwrap());
    //                 core::mem::drop(perm3.ok().unwrap());
    //                 core::mem::drop(perm4.ok().unwrap());
    //                 //kernel::task_exit();
    //             }
    //         }
    //     });
    //
    // /* blocking task */
    // Task::new()
    //     .priority(Priority(4))
    //     .static_stack(kernel::alloc_static_stack!(128))
    //     .spawn(move || {
    //         loop {
    //             cortex_m::asm::nop();
    //         }
    //     });


    let mut heartbeat = board.shield.led_1;
    Task::new()
        .priority(Priority(0))
        .static_stack(kernel::alloc_static_stack!(512))
        .spawn(move || {
            loop {
                kernel::sleep(200);
                heartbeat.toggle().ok();
            }
        });

    /* stack overflow */
    let mut led = board.shield.led_7;
    Task::new()
        .priority(Priority(1))
        .static_stack(kernel::alloc_static_stack!(512))
        .spawn(move || {
            loop {
                led.set_high().ok();
                kernel::sleep(1000);
                led.set_low().ok();
                kernel::sleep(200);

                match MUTEX.try_lock() {
                    Ok(mut v) => *v = 134,
                    Err(_) => {}
                };
                //recursion(1);
            }
        });

    sched::start();
}

#[allow(dead_code)]
fn recursion(a: u32) -> u32 {
    let b = a + 10;
    recursion(b)
}