// note(unsafe): Switching tasks is protected by critical sections. The compiler
// cannot verify critical section, thus they have to marked as unsafe.
#![allow(unsafe_code)]

use super::task::Task;
use super::linked_list::*;

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
use core::ptr::NonNull;
use core::cell::RefCell;
use core::mem::MaybeUninit;
use crate::boxed::Box;
use core::sync::atomic::{AtomicU32, Ordering};

type TaskPool = StaticListPool<Task, 16>;
static TASK_POOL: TaskPool = StaticListPool::new([None; 16]);

static mut SCHEDULER: Option<Scheduler> = None;

// todo: lock sched
// todo: replace with single linked list
pub struct Scheduler
{
    core: Peripherals,
    task_running: Option<Box<Node<Task>>>,
    task_idle: Option<Box<Node<Task>>>, // todo: remove option, there must always be an idle task
    tasks_ready: LinkedList<Task, TaskPool>,
    tasks_pending: LinkedList<Task, TaskPool>,
}

impl Scheduler
{
    pub fn init() {
        // init systick -> 1ms
        let mut core = Peripherals::take().unwrap();
        core.SYST.set_clock_source(SystClkSource::Core);
        // this is configured for the STM32F411 which has a default CPU clock of 48 MHz
        core.SYST.set_reload(48_000);
        core.SYST.clear_current();

        unsafe { SCHEDULER = Some(Scheduler {
            core,
            task_running: None,
            task_idle: None,
            tasks_ready: LinkedList::new(&TASK_POOL),
            tasks_pending: LinkedList::new(&TASK_POOL),
        })};
    }

    pub fn add(mut task: Task) {
         let scheduler = unsafe{ SCHEDULER.as_mut() }.unwrap();
         scheduler.tasks_ready.insert_back(task).ok();
    }

    pub fn replace_idle(mut task: Task) {

    }

    pub fn start() {
        let scheduler = unsafe{ SCHEDULER.as_mut() }.unwrap();

        scheduler.task_idle = scheduler.tasks_ready.pop_front();
        let task = scheduler.tasks_ready.pop_front();
        scheduler.task_running = task;

        scheduler.core.SYST.enable_counter();
        scheduler.core.SYST.enable_interrupt();

        // enable PendSV interrupt on lowest priority
        unsafe {
            scheduler.core.SCB.set_priority(scb::SystemHandler::PendSV, 0xFF);
        }

        // start first task
        unsafe {
            asm!(
            "msr   psp, {1}",     // set process stack pointer -> task stack
            "msr   control, {0}", // switch to thread mode
            "isb",                // recommended by ARM
            "pop   {{r4-r11}}",   // pop register we initialized
            "pop   {{r0-r3,r12,lr}}", // force function entry
            "pop   {{pc}}",       // 'jump' to the task entry function we put on the stack
            in(reg) 0x2,
            in(reg) scheduler.task_running.as_ref().unwrap().inner().stack_ptr() as u32,
            options(noreturn),
            );
        }
    }

    pub fn delay(ms: u32) {
        // todo: replace with system call
        // todo: unsafe -> already &mut in exec
        let scheduler = unsafe{ SCHEDULER.as_mut() }.unwrap();
        scheduler.task_running.as_mut().unwrap().inner_mut().delay(ms);

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
    scheduler.task_running.as_mut().unwrap().inner_mut().set_stack_ptr(psp as *mut usize);

    if scheduler.task_idle.is_some() {
        let pausing = scheduler.task_running.take().unwrap();
        if pausing.inner().next_wut() <= tick() { // todo: make more efficient with syscalls
            scheduler.tasks_ready.push_back(pausing);
        } else {
            scheduler.tasks_pending.push_back(pausing);
        }
    } else {
        scheduler.task_idle = scheduler.task_running.take();
    }

    // load next task
    scheduler.task_running = match scheduler.tasks_ready.pop_front() {
        Some(task) => Some(task),
        None => scheduler.task_idle.take(),
    };
    let psp = scheduler.task_running.as_ref().unwrap().inner().stack_ptr();
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
static COUNT: AtomicU32 = AtomicU32::new(0); // todo: replace with u64

#[exception]
fn SysTick() {
    COUNT.fetch_add(1, Ordering::Relaxed);
    let current = tick();

    // check if task is ready
    let scheduler = unsafe{ SCHEDULER.as_mut() }.unwrap();

    let mut cursor = scheduler.tasks_pending.cursor_front_mut();
    while let Some(task) = cursor.inner() {
        if task.next_wut() <= current as u64 {
            let node = cursor.take().unwrap();
            scheduler.tasks_ready.push_back(node);
            SCB::set_pendsv();
        }
        cursor.move_next();
    }
}

pub fn tick() -> u64 {
    COUNT.load(Ordering::Relaxed) as u64
}