// note(unsafe): Switching tasks is protected by critical sections. The compiler
// cannot verify critical section, thus they have to marked as unsafe.
#![allow(unsafe_code)]


use super::task::{Task, TaskError, Context};
use super::task_trait::TaskTrait;
use core::borrow::Borrow;
use cortex_m::peripheral::{
    Peripherals,
    SYST,
    syst::SystClkSource,
};
use cortex_m_rt::exception;

pub struct Scheduler<F,C>
    where F: FnMut(&mut C) -> Result<(), TaskError>, C: Context
{
    tasks: [Option<Task<F,C>>; 5],
    syst: SYST
}

impl<F,C> Scheduler<F,C>
    where F: FnMut(&mut C) -> Result<(), TaskError>, C: Context
{
    pub fn new() -> Scheduler<F,C> {
        // init systick -> 1ms
        let mut syst = Peripherals::take().unwrap().SYST;
        syst.set_clock_source(SystClkSource::Core);
        // this is configured for the STM32F411 which has a default CPU clock of 48 MHz
        syst.set_reload(48_000);
        syst.clear_current();
        syst.enable_counter();
        syst.enable_interrupt();

        Scheduler {
            tasks: [None, None, None, None, None],
            syst: syst,
        }
    }

    pub fn spawn(&mut self, task: Task<F,C>) {
        let tasks = &mut self.tasks;
        for _task in tasks {
            if _task.is_none() {
                *_task = Some(task);
                break;
            }
        }
    }

    pub fn exec(&mut self) {
        for task in &mut self.tasks {
            if task.is_some() {
                let current_task = task.as_mut().unwrap();
                if current_task.get_next_wut() <= get_tick() {
                    current_task.run();
                }
            }
        }
    }

    pub fn delay(&self, ms: u32) {
        let end = get_tick() + ms;
        while get_tick() < end {

        }
    }
}


static mut COUNT: u32 = 0;

#[exception]
fn SysTick() {
    // `COUNT` has transformed to type `&mut u32` and it's safe to use
    unsafe { COUNT += 1; }
}

pub fn get_tick() -> u32 {
    unsafe { COUNT }
}