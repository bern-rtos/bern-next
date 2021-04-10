//#![deny(unsafe_code)] todo: just for now
#![no_main]
#![no_std]


use bern_kernel::{
    task::Task,
    scheduler::Scheduler,
};


// dev profile: easier to debug panics; can put a breakpoint on `rust_begin_unwind`
#[cfg(debug_assertions)]
use panic_halt as _;
//use panic_probe as _;

// release profile: minimize the binary size of the application
#[cfg(not(debug_assertions))]
use panic_abort as _;

// Halt on panic and print the stack trace to RTT
//use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m;
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;
use crate::hal::{prelude::*, stm32};

#[entry]
fn main() -> ! {
    rtt_init_print!(BlockIfFull);
    rprintln!("Hello, world!");

    Scheduler::init();
    /* idle task */
    Task::spawn(move | | {
        loop {
            cortex_m::asm::nop();
        }
    },
                bern_kernel::alloc_static_stack!(128)
    );

    // Take hardware peripherals
    let stm32_peripherals = stm32::Peripherals::take().expect("cannot take stm32 peripherals");

    // delay
    // Set up the system clock. We want to run at 48MHz for this one.
    let rcc = stm32_peripherals.RCC.constrain();
    let _clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

    // gpio's
    let gpioa = stm32_peripherals.GPIOA.split();
    let _gpiob = stm32_peripherals.GPIOB.split();
    let gpioc = stm32_peripherals.GPIOC.split();

    // itm output
    //let stim = &mut cortex_peripherals.ITM.stim[1];

    // button to led map module
    let mut led = gpioa.pa5.into_push_pull_output();
    //let mut led_0 = gpiob.pb11.into_push_pull_output();
    //let mut led_1 = gpiob.pb12.into_push_pull_output();
    //let mut led_2 = gpioc.pc2.into_push_pull_output();
    //let mut led_3 = gpioc.pc3.into_push_pull_output();
    //let mut led_4 = gpioa.pa2.into_push_pull_output();
    //let mut led_5 = gpioa.pa3.into_push_pull_output();
    let mut led_6 = gpioc.pc6.into_push_pull_output();
    let mut led_7 = gpioc.pc7.into_push_pull_output();
    //let button = gpioc.pc13.into_floating_input();

    /* task 1 */
    Task::spawn(move | | {
        led.set_high();
        loop {
            led_7.toggle();
            Scheduler::delay(100);
        }
    },
                bern_kernel::alloc_static_stack!(256)
    );

    /* task 2 */
    let mut a = 0;
    Task::spawn(move | | {
        loop {
            a += 1;
            led_6.set_high();
            Scheduler::delay(50);
            led_6.set_low();
            Scheduler::delay(400);
            //rprintln!("Hello, world!");
        }
    },
                bern_kernel::alloc_static_stack!(256)
    );

    Scheduler::start();
    loop {
        panic!("We should have never arrived here!");
    }
}