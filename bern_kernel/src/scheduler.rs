// note(unsafe): Switching tasks is protected by critical sections. The compiler
// cannot verify critical section, thus they have to marked as unsafe.
#![allow(unsafe_code)]

// `use crate::` is confusing CLion
use crate::task::Task;
use crate::time;
use crate::collection::linked_list::*;
use crate::collection::boxed::Box;
use crate::sync::simple_mutex::SimpleMutex;

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
    tasks_suspended: LinkedList<Task, TaskPool>,
    tasks_terminated: LinkedList<Task, TaskPool>,
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
                tasks_suspended: LinkedList::new(&TASK_POOL),
                tasks_terminated: LinkedList::new(&TASK_POOL),
            });
        } else {
            // todo: we're screwed
        }
        SCHEDULER.release();
    }

    pub fn add(mut task: Task) {
        if let Some(sched) = SCHEDULER.lock() {
            sched.as_mut().unwrap().tasks_ready.emplace_back(task).ok();
        } else {
            // todo
        }
        SCHEDULER.release();
    }

    pub fn replace_idle(mut task: Task) {

    }

    pub fn start() -> ! {
        let mut sched = match SCHEDULER.lock() {
            Some(sched) => sched.as_mut().unwrap(),
            None => panic!("oops"), // todo: error handling
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

    pub fn sleep(ms: u32) {
        let mut sched = match SCHEDULER.lock() {
            Some(sched) => sched.as_mut().unwrap(),
            None => return, // todo: error handling
        };

        sched.task_running.as_mut().unwrap().inner_mut().sleep(ms);
        SCHEDULER.release();

        SCB::set_pendsv();
    }

    pub fn task_terminate() {
        let mut sched = match SCHEDULER.lock() {
            Some(sched) => sched.as_mut().unwrap(),
            None => return, // todo: error handling
        };
    }

    pub fn schedule() {
        let now = time::tick();
        let mut sched = match SCHEDULER.lock() {
            Some(sched) => sched.as_mut().unwrap(),
            None => return, // todo: error handling
        };
        // update pending -> ready list
        let mut trigger_switch = false;
        let mut cursor = sched.tasks_suspended.cursor_front_mut();
        while let Some(task) = cursor.inner() {
            if task.next_wut() <= now as u64 {
                // todo: this is inefficient, we know that node exists
                if let Some(node) = cursor.take() {
                    sched.tasks_ready.push_back(node);
                    trigger_switch = true;
                }
            } else {
                break; // the is sorted, we can abort early
            }
            cursor.move_next();
        }

        SCHEDULER.release();
        if trigger_switch {
            SCB::set_pendsv();
        }
    }
}


pub fn idle() {
    loop {
        asm::nop();
    }
}

////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
#[naked] // todo: move to separate assembly file and introduce at link time
extern "C" fn PendSV() {
    // Source: Definitive Guide to Cortex-M3/4, p. 342
    // store stack of current task
    unsafe {
        asm!(
            "mrs   r0, psp",
            "stmdb r0!, {{r4-r11}}",
            "push  {{lr}}",
            "bl    switch_task",
            "pop   {{lr}}",
            "mov   r3, #2",        // todo: read from function
            "msr   control, r3",   // switch to thread mode
            "ldmia r0!, {{r4-r11}}",
            "msr   psp, r0",
            "bx    lr",
            options(noreturn),
        )
    }
}

#[no_mangle]
fn switch_task(psp: u32) -> u32 {
    let mut sched = match SCHEDULER.lock() {
        Some(sched) => sched.as_mut().unwrap(),
        None => return 0, // todo: error handling
    };
    sched.task_running.as_mut().unwrap().inner_mut().set_stack_ptr(psp as *mut usize);

    if sched.task_idle.is_some() {
        let pausing = sched.task_running.take().unwrap();
        if pausing.inner().next_wut() <= time::tick() { // todo: make more efficient with syscalls
            sched.tasks_ready.push_back(pausing);
        } else {
            sched.tasks_suspended.insert_when(pausing, | pausing, task | {
                pausing.next_wut() < task.next_wut()
            });
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
    psp as u32
}