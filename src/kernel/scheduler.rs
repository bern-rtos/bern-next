// note(unsafe): Switching tasks is protected by critical sections. The compiler
// cannot verify critical section, thus they have to marked as unsafe.
#![allow(unsafe_code)]


use super::task::{Task, TaskError, Context};
use cortex_m::peripheral::{
    Peripherals,
    SYST,
    syst::SystClkSource,
};
use cortex_m_rt::exception;

pub struct Scheduler<'a>
{
    tasks: [Option<Task<'a>>; 5],
    syst: SYST,
}

impl<'a> Scheduler<'a>
{

    pub fn new() -> Self {
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

    pub fn spawn(&mut self, task: Task<'a>) {
        for _task in self.tasks.iter_mut() {
            if _task.is_none() {
                *_task = Some(task);
                break;
            }
        }
    }

    pub fn exec(&mut self) {
        for task in self.tasks.iter_mut() {
            if task.is_some() {
                if task.as_mut().unwrap().get_next_wut() < get_tick() {
                    task.as_mut().unwrap().run();
                }
            }
        }
    }
    //
    // fn yield_sched(&mut self) {
    //     self.exec();
    // }
}


////////////////////////////////////////////////////////////////////////////////
static mut COUNT: u64 = 0;

#[exception]
fn SysTick() {
    // `COUNT` has transformed to type `&mut u32` and it's safe to use
    unsafe { COUNT += 1; }
}

pub fn get_tick() -> u64 {
    unsafe { COUNT }
}