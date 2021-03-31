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
use crate::kernel::task::{Context, Runnable};
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
    let button = gpioc.pc13.into_floating_input();

    /* task 1 */
    task_spawn(move | | {
        led.toggle();
        Ok(())
    });

    loop {
        Scheduler::exec();
    }
}

fn task_spawn<F>(closure: F)
    where
        F: 'static + Sync + FnMut() -> Result<(), TaskError>,
{
    let mut runnable = RunnableClosure::new(closure);
    let mut task = Task::new(
        kernel::boxed::Box::new(runnable, unsafe { TASK_A_STACK.as_mut() })
    );
    Scheduler::add(task);
}

static mut TASKS: Option<Task> = None;
static mut TASK_A_STACK: [u8; 256] = [0; 256];