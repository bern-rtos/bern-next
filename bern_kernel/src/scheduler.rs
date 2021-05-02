// note(unsafe): Switching tasks is protected by critical sections. The compiler
// cannot verify critical section, thus they have to marked as unsafe.
#![allow(unsafe_code)]


/// # Basic Concept
/// Keep interrupt latency as short as possible, move work to PendSV.


use core::sync::atomic::{self, Ordering};

use crate::task::{Task, Transition};
use crate::time;
use crate::collection::linked_list::*;
use crate::collection::boxed::Box;
use crate::sync::simple_mutex::SimpleMutex;

use bern_arch::{Core, ContextSwitch};
use bern_arch::arch::{ArchCore, Arch};


type TaskPool = StaticListPool<Task, 16>;
static TASK_POOL: TaskPool = StaticListPool::new([None; 16]);

static SCHEDULER: SimpleMutex<Option<Scheduler>> = SimpleMutex::new(None);

pub struct Scheduler
{
    core: ArchCore,
    task_running: Option<Box<Node<Task>>>,
    task_idle: Option<Box<Node<Task>>>,
    tasks_ready: LinkedList<Task, TaskPool>,
    tasks_suspended: LinkedList<Task, TaskPool>,
    tasks_terminated: LinkedList<Task, TaskPool>,
}

impl Scheduler
{
    pub fn init() {
        let core = ArchCore::new();

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
            panic!("Scheduler already locked, init called at wrong place");
        }
        SCHEDULER.release();
    }


    pub fn start() -> ! {
        let mut sched = match SCHEDULER.lock() {
            Some(sched) => sched.as_mut().unwrap(),
            None => panic!("Scheduler already locked, (todo reentrant scheduler)"),
        };

        sched.task_idle = sched.tasks_ready.pop_front();
        let task = sched.tasks_ready.pop_front();
        sched.task_running = task;

        sched.core.start();

        let stack_ptr = sched.task_running.as_ref().unwrap().inner().stack_ptr();
        SCHEDULER.release();

        Arch::start_first_task(stack_ptr);
    }

    pub fn add(task: Task) {
        match SCHEDULER.lock() {
            Some(sched) => {
                sched.as_mut().unwrap().tasks_ready.emplace_back(task).ok();
                SCHEDULER.release();
            },
            None => panic!("Scheduler already locked, (todo reentrant scheduler)"),
        };
    }

    pub fn replace_idle(_task: Task) {

    }


    pub fn sleep(ms: u32) {
        let sched = match SCHEDULER.lock() {
            Some(sched) => sched.as_mut().unwrap(),
            None => panic!("Scheduler already locked, (todo reentrant scheduler)"),
        };

        let task = sched.task_running.as_mut().unwrap().inner_mut();
        task.sleep(ms);
        task.set_transition(Transition::Suspending);
        SCHEDULER.release();

        Arch::trigger_context_switch();
    }

    pub fn task_terminate() {
        let sched = match SCHEDULER.lock() {
            Some(sched) => sched.as_mut().unwrap(),
            None => panic!("Scheduler already locked, (todo reentrant scheduler)"),
        };

        let task = sched.task_running.as_mut().unwrap().inner_mut();
        task.set_transition(Transition::Terminating);
        SCHEDULER.release();

        Arch::trigger_context_switch();
    }

    pub fn tick_update() {
        let now = time::tick();
        let sched = match SCHEDULER.lock() {
            Some(sched) => sched.as_mut().unwrap(),
            None => panic!("Scheduler already locked, (todo reentrant scheduler)"),
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
                break; // the list is sorted by wake-up time, we can abort early
            }
            cursor.move_next();
        }

        SCHEDULER.release();
        if trigger_switch {
            Arch::trigger_context_switch();
        }
    }
}


pub fn idle() {
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

////////////////////////////////////////////////////////////////////////////////

/// This function must be called from the architecture specific task switch
/// implementation.
#[no_mangle]
fn switch_context(psp: u32) -> u32 {
    let mut sched = match SCHEDULER.lock() {
        Some(sched) => sched.as_mut().unwrap(),
        None => panic!("Scheduler already locked, (todo reetrant scheduler)"),
    };
    sched.task_running.as_mut().unwrap().inner_mut().set_stack_ptr(psp as *mut usize);

    if sched.task_idle.is_some() {
        let mut pausing = sched.task_running.take().unwrap();
        match pausing.inner().transition() {
            Transition::None => sched.tasks_ready.push_back(pausing),
            Transition::Suspending => {
                pausing.inner_mut().set_transition(Transition::None);
                sched.tasks_suspended.insert_when(pausing, | pausing, task | {
                    pausing.next_wut() < task.next_wut()
                });
            },
            Transition::Terminating => {
                pausing.inner_mut().set_transition(Transition::None);
                sched.tasks_terminated.push_back(pausing);
            },
            _ => (),
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