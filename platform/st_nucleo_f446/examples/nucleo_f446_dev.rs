//#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m;
use cortex_m_rt::entry;
use st_nucleo_f446::StNucleoF446;
use stm32f4xx_hal::prelude::*;

use bern_kernel as kernel;
use kernel::{
    task::Priority,
    sched,
    process::Process,
};

#[link_section=".process.my_process"]
static SOME_ARR: [u8; 8] = [1,2,3,4,5,6,7,8];

static PROC: &Process = kernel::new_process!(my_process, 4096);
static IDLE_PROC: &Process = kernel::new_process!(idle, 1024);


#[entry]
fn main() -> ! {
    cortex_m::asm::bkpt();
    let mut board = StNucleoF446::new();

    sched::init();
    sched::set_tick_frequency(
        1_000,
        48_000_000
    );

    let mut heartbeat = board.led.take().unwrap();

    PROC.create_thread()
        .priority(Priority(0))
        .stack(1024)
        .spawn(move || {

            loop {
                heartbeat.set_high().ok();
                kernel::sleep(100);
                heartbeat.set_low().ok();
                kernel::sleep(900);

                //defmt::info!("Hello from task A! {}", i);
                //if unsafe { AAA }  % 2 == 0 && unsafe { FF } == 123 {
                //    i += 1;
                //}

                //recursion(1);
            }
        });

    IDLE_PROC.create_thread()
        .idle_task()
        .stack(512)
        .spawn(move || {
            loop {

            }
        });


    /*Task::new()
        .priority(Priority(0))
        .stack(kernel::mem::size_from_raw!(1024))
        .spawn(move || {
            let mut b = 0;
            loop {
                kernel::sleep(200);

                //defmt::info!("Hello from task B! {}", b);
                b += 1;
            }
        });*/

    sched::start();
}

fn recursion(a: u32) {
    recursion(a + 1);
}