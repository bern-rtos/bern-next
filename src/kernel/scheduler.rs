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

pub struct TaskList<F1,F2>
where
    F1: FnMut(&mut Context) -> Result<(), TaskError>,
    F2: FnMut(&mut Context) -> Result<(), TaskError>,
{
    pub task_1: Option<Task<F1>>,
    pub task_2: Option<Task<F2>>,
}

pub struct Scheduler<F1,F2>
where
    F1: FnMut(&mut Context) -> Result<(), TaskError>,
    F2: FnMut(&mut Context) -> Result<(), TaskError>,
{
    tasks: TaskList<F1,F2>,
    syst: SYST,
}

impl<F1,F2> Scheduler<F1,F2>
where
    F1: FnMut(&mut Context) -> Result<(), TaskError>,
    F2: FnMut(&mut Context) -> Result<(), TaskError>,
{

    pub fn new(tasks: TaskList<F1,F2>) -> Scheduler<F1,F2> {
        // init systick -> 1ms
        let mut syst = Peripherals::take().unwrap().SYST;
        syst.set_clock_source(SystClkSource::Core);
        // this is configured for the STM32F411 which has a default CPU clock of 48 MHz
        syst.set_reload(48_000);
        syst.clear_current();
        syst.enable_counter();
        syst.enable_interrupt();

        Scheduler {
            tasks: tasks,
            syst: syst,
        }
    }

    pub fn exec(&mut self) {
        let task = &mut self.tasks.task_1;
        if task.is_some() {
            if task.as_mut().unwrap().get_next_wut() < get_tick() {
                task.as_mut().unwrap().run();
            }
        }

        let task = &mut self.tasks.task_2;
        if task.is_some() {
            if task.as_mut().unwrap().get_next_wut() < get_tick() {
                task.as_mut().unwrap().run();
            }
        }
    }

    fn yield_sched(&mut self) {
        self.exec();
    }
}

////////////////////////////////////////////////////////////////////////////////

//struct SchedulerInstance<F>
//    where F: FnMut(&mut Context) -> Result<(), TaskError>
//{
//    scheduler: Option<Scheduler<F>>,
//}

//static mut SCHEDULER: Option<Scheduler<F>> = None;

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