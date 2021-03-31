#![feature(unsize)]
//#![deny(unsafe_code)] todo: just for now
#![no_main]
#![no_std]

mod kernel;

use kernel::{
    task::Task,
    task::TaskError,
    task::RunnableClosure,
    scheduler::Scheduler,
    scheduler,
};

#[allow(unused_extern_crates)]
// Halt on panic and print the stack trace to SWO
extern crate panic_itm;

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;
use crate::hal::{prelude::*, stm32};
use embedded_hal;
use crate::kernel::task::Runnable;
use core::mem::take;
use core::pin::Pin;

#[entry]
fn main() -> ! {
    Scheduler::init();

    // Take hardware peripherals
    let stm32_peripherals = stm32::Peripherals::take().expect("cannot take stm32 peripherals");

    // delay
    // Set up the system clock. We want to run at 48MHz for this one.
    let rcc = stm32_peripherals.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

    // gpio's
    let gpioa = stm32_peripherals.GPIOA.split();
    let gpiob = stm32_peripherals.GPIOB.split();
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
    //let mut led_6 = gpioc.pc6.into_push_pull_output();
    let mut led_7 = gpioc.pc7.into_push_pull_output();
    //let button = gpioc.pc13.into_floating_input();

    /* task 1 */
    Task::spawn(move | | {
            led.toggle();
            led_7.toggle();
            Scheduler::delay(500);
            Ok(())
        },
        alloc_static_stack!(256)
    );

    loop {
        Scheduler::exec();
    }
}
