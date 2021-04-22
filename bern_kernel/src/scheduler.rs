// note(unsafe): Switching tasks is protected by critical sections. The compiler
// cannot verify critical section, thus they have to marked as unsafe.
#![allow(unsafe_code)]

// `use crate::` is confusing CLion
use super::task::Task;
use super::collection::linked_list::*;
use super::collection::boxed::Box;
use super::sync::simple_mutex::SimpleMutex;

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
use core::sync::atomic::{AtomicU32, Ordering};
use crate::task::{StackFrameExtension, StackFrameException};
use core::mem;

type TaskPool = StaticListPool<Task, 16>;
static TASK_POOL: TaskPool = StaticListPool::new([None; 16]);

static SCHEDULER: SimpleMutex<Option<Scheduler>> = SimpleMutex::new(None);

pub struct Scheduler
{
    core: Peripherals,
    task_running: Option<Box<Node<Task>>>,
    task_idle: Option<Box<Node<Task>>>,
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

        if let Some(sched) = SCHEDULER.lock() {
            sched.replace(Scheduler {
                core,
                task_running: None,
                task_idle: None,
                tasks_ready: LinkedList::new(&TASK_POOL),
                tasks_pending: LinkedList::new(&TASK_POOL),
            });
        } else {
            // todo: we're screwed
        }
        SCHEDULER.release();
    }

    pub fn add(mut task: Task) {
        if let Some(sched) = SCHEDULER.lock() {
            sched.as_mut().unwrap().tasks_ready.insert_back(task).ok();
        } else {
            // todo
        }
        SCHEDULER.release();
    }

    pub fn replace_idle(mut task: Task) {

    }

    pub fn start() {
        let mut sched = match SCHEDULER.lock() {
            Some(sched) => sched.as_mut().unwrap(),
            None => return, // todo: error handling
        };

        sched.task_idle = sched.tasks_ready.pop_front();
        let task = sched.tasks_ready.pop_front();
        sched.task_running = task;

        sched.core.SYST.enable_counter();
        sched.core.SYST.enable_interrupt();

        // enable PendSV interrupt on lowest priority
        unsafe {
            sched.core.SCB.set_priority(scb::SystemHandler::PendSV, 0xFF);
        }

        let stack_ptr = sched.task_running.as_ref().unwrap().inner().stack_ptr();
        SCHEDULER.release();
        // start first task
        unsafe {
            asm!(
            "msr   psp, {1}",       // set process stack pointer -> task stack
            "msr   control, {0}",   // switch to thread mode
            "isb",                  // recommended by ARM
            "pop   {{r4-r11}}",     // pop register we initialized
            "pop   {{r0-r3,r12,lr}}", // force function entry
            "pop   {{pc}}",         // 'jump' to the task entry function we put on the stack
            in(reg) 0x2,            // privileged task
            in(reg) stack_ptr as u32,
            options(noreturn),
            );
        }
    }

    pub fn delay(ms: u32) {
        // todo: replace with system call
        // todo: unsafe -> already &mut in exec
        let mut sched = match SCHEDULER.lock() {
            Some(sched) => sched.as_mut().unwrap(),
            None => return, // todo: error handling
        };

        sched.task_running.as_mut().unwrap().inner_mut().delay(ms);
        SCHEDULER.release();

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
    let mut sched = match SCHEDULER.lock() {
        Some(sched) => sched.as_mut().unwrap(),
        None => return, // todo: error handling
    };
    sched.task_running.as_mut().unwrap().inner_mut().set_stack_ptr(psp as *mut usize);

    if sched.task_idle.is_some() {
        let pausing = sched.task_running.take().unwrap();
        if pausing.inner().next_wut() <= tick() { // todo: make more efficient with syscalls
            sched.tasks_ready.push_back(pausing);
        } else {
            sched.tasks_pending.push_back(pausing);
        }
    } else {
        sched.task_idle = sched.task_running.take();
    }

    // load next task
    sched.task_running = match sched.tasks_ready.pop_front() {
        Some(task) => Some(task),
        None => sched.task_idle.take(),
    };
    let psp = sched.task_running.as_ref().unwrap().inner().stack_ptr();
    SCHEDULER.release();
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
    unsafe {asm!(
        "bl syscall_handler",
        //"TST lr, #4",
        //"ITE EQ",
        //"MRSEQ r2, MSP",
        //"MRSNE r2, PSP",
        //"LDR r3, [r2, #5]",
        //"bl syscall_handler",
    )};
}

#[no_mangle]
fn syscall_handler(service: u8, r1: u32, r2: u32) {
    match service {
        1 => Scheduler::delay(r1),
        2 => {
            let task: Task = unsafe { mem::transmute_copy(&*(r1 as *const Task)) };
            Scheduler::add(task);
        },
        42 => {
            asm::bkpt()
        },
        _ => asm::bkpt(),
    }
}

////////////////////////////////////////////////////////////////////////////////

static COUNT: AtomicU32 = AtomicU32::new(0); // todo: replace with u64

#[exception]
fn SysTick() {
    COUNT.fetch_add(1, Ordering::Relaxed);
    let current = tick();


    let mut sched = match SCHEDULER.lock() {
        Some(sched) => sched.as_mut().unwrap(),
        None => return, // todo: error handling
    };
    // update pending -> ready list
    let mut cursor = sched.tasks_pending.cursor_front_mut();
    let mut new_read = false;
    while let Some(task) = cursor.inner() {
        // todo: sort list so we don't have to look through the whole list
        if task.next_wut() <= current as u64 {
            if let Some(node) = cursor.take() {
                sched.tasks_ready.push_back(node);
                new_read = true;
            }
        }
        cursor.move_next();
    }
    SCHEDULER.release();
    if new_read {
        SCB::set_pendsv();
    }
}

pub fn tick() -> u64 {
    COUNT.load(Ordering::Relaxed) as u64
}