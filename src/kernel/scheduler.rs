// note(unsafe): Switching tasks is protected by critical sections. The compiler
// cannot verify critical section, thus they have to marked as unsafe.
#![allow(unsafe_code)]


use super::task::Task;
use super::task_trait::TaskTrait;
use core::borrow::Borrow;
use cortex_m::peripheral::{
    Peripherals,
    SYST,
    syst::SystClkSource,
};
use cortex_m_rt::exception;

//pub struct Scheduler {
pub struct Scheduler<T> {
    //tasks: [Option<Task>; 5],
    tasks: [Option<T>; 5],
    syst: SYST
}

impl<T> Scheduler<T> where T: TaskTrait {
    pub fn new() -> Scheduler<T> {
        let mut syst = Peripherals::take().unwrap().SYST;
        // configures the system timer to trigger a SysTick exception every second
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

    //pub fn spawn(task: Task) {
    pub fn spawn(&mut self, task: T) {
        // note(unsafe)
        //let tasks = unsafe { &mut SCHEDULER.tasks };
        let tasks = &mut self.tasks;
        for _task in tasks {
            if _task.is_none() {
                *_task = Some(task);
                break;
            }
        }
    }

    //pub fn exec() {
    pub fn exec(&mut self) {
        //let tasks = unsafe { &SCHEDULER.tasks };
        //let tasks = &mut self.tasks;
        for task in &mut self.tasks {
            if task.is_some() {
                task.as_mut().unwrap().run();
            }
        }
    }

    pub fn delay(&self, ms: u32) {
        let end = unsafe { COUNT } + ms;
        while unsafe{ COUNT < end } {

        }
    }
}


// requires unsafe
//static mut SCHEDULER: Scheduler = Scheduler {
//    tasks: [None, None, None, None, None],
//};
static mut COUNT: u32 = 0;

#[exception]
fn SysTick() {


    // `COUNT` has transformed to type `&mut u32` and it's safe to use
    unsafe { COUNT += 1; }
}
