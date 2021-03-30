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
use crate::kernel::task::Context;
use core::borrow::BorrowMut;
use core::pin::Pin;

#[entry]
fn main() -> ! {
    // Take hardware peripherals
    let stm32_peripherals = stm32::Peripherals::take().expect("cannot take stm32 peripherals");

    // delay
    // Set up the system clock. We want to run at 48MHz for this one.
    let rcc = stm32_peripherals.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

    // gpio's
    //let gpioa = stm32_peripherals.GPIOA.split();
    let gpiob = stm32_peripherals.GPIOB.split();
    let gpioc = stm32_peripherals.GPIOC.split();

    // itm output
    //let stim = &mut cortex_peripherals.ITM.stim[1];

    // button to led map module
    //let mut led = gpioa.pa5.into_push_pull_output();
    let button = gpioc.pc13.into_floating_input();

    /* task 1 */
    let mut c = 10;
    let mut button_closure = RunnableClosure::new( move | | {
        //button;
        c += 1;
        Ok(())
    });
    let mut vla = pin_this(&mut button_closure.runnable);
    let basfd = vla.as_mut().get_mut();
    basfd();
    let mut task2 = Task::new(basfd);

    //RunnableClosure::run(&mut button_closure);
    /* todo: implement some sort of static Box<> type */
    /*let mut runnable = RunnableClosure::new(move |c| {
        led.toggle();
        c.delay(100);
        Ok(())
    });
    let mut task1 = Task::new(&mut runnable);*/

    /* task 2 */
    /*let mut runnable = RunnableClosure::new(move |c| {
        Ok(())
    });
    let mut task2 = Task::new(&mut runnable);*/

    //let mut task1 = Task::new(&mut a_task);

    Scheduler::init();
    //Scheduler::add(task1);
    //let mut scheduler = Scheduler::new();
    //scheduler.spawn(task1);
    //scheduler.spawn(task2);
    //scheduler.spawn(new_task());

    loop {
        Scheduler::exec();
    }
}

// fn new_task<'a>() -> Task<'a> {
//     /* task 2 */
//     let mut runnable = RunnableClosure::new(move |c| {
//         Ok(())
//     });
//     Task::new(&mut runnable)
// }


fn a_task() -> Result<(), TaskError> {
    // this is very stupid, but it will do for now
    let stm32_peripherals = unsafe{ stm32::Peripherals::steal() };
    let gpioa = stm32_peripherals.GPIOA.split();
    let mut led = gpioa.pa5.into_push_pull_output();

    loop {
        led.toggle();
    }

    Ok(())
}

fn pin_this<F>(closure: &'static mut F) -> Pin<&'static mut F> where F: 'static + Sync + FnMut() -> Result<(), TaskError>,
{
    unsafe { Pin::new_unchecked(closure) }
}