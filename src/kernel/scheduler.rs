// note(unsafe): Switching tasks is protected by critical sections. The compiler
// cannot verify critical section, thus they have to marked as unsafe.
#![allow(unsafe_code)]

use super::task::{Task, TaskError};
use cortex_m::peripheral::{
    syst::SystClkSource,
};
use cortex_m::{
    asm,
    register::*,
    peripheral::*,
    interrupt::*,
};
use cortex_m_rt::exception;

static mut SCHEDULER: Option<Scheduler> = None;

// todo: lock sched
// todo: replace with single linked list
pub struct Scheduler<'a>
{
    tasks: [Option<Task<'a>>; 5],
    core: Peripherals,
    current_task: Option<*mut Task<'a>>, // todo: I'm not fighting the borrow checker until I use a linked list
    next_task: Option<*mut Task<'a>>,
}

impl<'a> Scheduler<'a>
{
    pub fn init() {
        // init systick -> 1ms
        let mut core = Peripherals::take().unwrap();
        core.SYST.set_clock_source(SystClkSource::Core);
        // this is configured for the STM32F411 which has a default CPU clock of 48 MHz
        core.SYST.set_reload(48_000);
        core.SYST.clear_current();

        unsafe { SCHEDULER = Some(Scheduler {
                tasks: [None, None, None, None, None],
                core,
                current_task: None,
                next_task: None,
            })
        };
    }

    pub fn add(mut task: Task<'static>) {
        let scheduler = unsafe{ SCHEDULER.as_mut() }.unwrap();

        for _task in scheduler.tasks.iter_mut() {
            if _task.is_none() {
                *_task = Some(task);
                break;
            }
        }
    }

    pub fn start() {
        let scheduler = unsafe{ SCHEDULER.as_mut() }.unwrap();
        scheduler.current_task = Some(scheduler.tasks[1].as_mut().unwrap());
        scheduler.next_task = Some(scheduler.tasks[0].as_mut().unwrap());

        scheduler.core.SYST.enable_counter();
        scheduler.core.SYST.enable_interrupt();

        // enable PendSV interrupt on lowest priority
        unsafe {
            scheduler.core.SCB.set_priority(scb::SystemHandler::PendSV, 0xFF);
        }

        // start first task
        unsafe {
            asm!(
            "msr   psp, {1}", // set process stack pointer -> task stack
            "msr   control, {0}", // switch to thread mode
            "isb", // recommended by ARM
            "pop   {{r4-r11}}", // pop register we initialized
            "pop   {{r0-r3,r12,lr}}", // force function entry
            "pop   {{pc}}", // 'jump' to the task entry function we put on the stack
            in(reg) 0x2,
            in(reg) scheduler.current_task.as_mut().unwrap().as_mut().unwrap().get_psp() as u32,
            options(noreturn),
            );
        }
    }

    pub fn delay(ms: u32) {
        // todo: replace with system call
        // todo: unsafe -> already &mut in exec
        let scheduler = unsafe{ SCHEDULER.as_mut() }.unwrap();
        unsafe { scheduler.current_task.as_mut().unwrap().as_mut().unwrap() }.delay(ms);
        unsafe { scheduler.current_task.as_mut().unwrap().as_mut().unwrap() }.get_psp();

        SCB::set_pendsv();
    }
}


pub fn idle() {
    loop {
        asm::nop();
    }
}

////////////////////////////////////////////////////////////////////////////////

#[exception]
fn PendSV() {

    // Source: Definitive Guide to Cortex-M3/4, p. 342
    // store stack of current task
    let mut psp: u32;
    unsafe {
        asm!(
            "mrs   r0, psp",
            "stmdb r0!, {{r4-r11}}",
            out("r0") psp
        );
    }
    let scheduler = unsafe{ SCHEDULER.as_mut() }.unwrap();
    unsafe { scheduler.current_task.as_mut().unwrap().as_mut().unwrap() }.set_psp(psp as *mut usize);

    // load next task
    let task = scheduler.next_task.take().unwrap();
    let psp = unsafe { task.as_mut().unwrap() }.get_psp();
    scheduler.current_task = Some(task);
    scheduler.next_task = Some(scheduler.tasks[0].as_mut().unwrap()); // if in doubt -> idle
    unsafe {
        asm!(
            "ldmia r0!, {{r4-r11}}",
            "msr   psp, r0",
            in("r0") psp as u32,
        )
    }
}

#[exception]
fn SVCall() {
    asm::bkpt();
}

////////////////////////////////////////////////////////////////////////////////
static mut COUNT: u64 = 0;

#[exception]
fn SysTick() {
    // `COUNT` has transformed to type `&mut u32` and it's safe to use
    unsafe { COUNT += 1; }

    // check if task is ready
    let scheduler = unsafe{ SCHEDULER.as_mut() }.unwrap();
    let task = scheduler.tasks[1].as_mut().unwrap();
    for (i, task) in scheduler.tasks.iter_mut().enumerate() {
        if i == 0 || task.is_none() {
            continue; // idle task
        }
        if task.as_mut().unwrap().get_next_wut() <= unsafe { COUNT } {
            // todo: find better comparison between tasks
            if task.as_mut().unwrap().get_psp() != unsafe { scheduler.current_task.as_mut().unwrap().as_mut().unwrap() }.get_psp() {
                scheduler.next_task = Some(task.as_mut().unwrap());
                SCB::set_pendsv();
            }
        }
    }
}

pub fn get_tick() -> u64 {
    unsafe { COUNT }
}