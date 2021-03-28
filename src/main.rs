#![deny(unsafe_code)]
#![no_main]
#![no_std]

mod kernel;
mod task_1;

use kernel::{
    task::Task,
    task::TaskError,
    scheduler::Scheduler,
    scheduler,
    task_trait::TaskTrait,
};

#[allow(unused_extern_crates)]
// Halt on panic and print the stack trace to SWO
extern crate panic_itm;

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;
use crate::hal::{prelude::*, stm32};
use embedded_hal;

#[entry]
fn main() -> ! {
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



    let task1 = Task::new(move |c| {
        some_runnable(&mut led);
        c.delay(500);
        Ok(())
    });

    let task2 = Task::new(move |c| {
        for i in 0..100 {

        }
        Ok(())
    });
    //bla(&some_runnable);

    let tasks = scheduler::TaskList{
        task_1: Some(task1),
        task_2: Some(task2),
    };
    let mut scheduler = Scheduler::new(tasks);

    loop {
        scheduler.exec();
    }
}

fn some_runnable<Led>(led: &mut Led) -> Result<(), TaskError>
    where Led: embedded_hal::digital::v2::ToggleableOutputPin
{
    led.toggle();
    Ok(())
}

fn bla(fun: &fn() -> Result<(), TaskError>) {

}
