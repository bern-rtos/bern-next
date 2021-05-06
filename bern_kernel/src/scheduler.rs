// note(unsafe): Switching tasks is protected by critical sections. The compiler
// cannot verify critical section, thus they have to marked as unsafe.
#![allow(unsafe_code)]


/// # Basic Concept
/// Keep interrupt latency as short as possible, move work to PendSV.


use core::sync::atomic::{self, Ordering};
use arr_macro::arr;

use crate::task::{self, Task, Transition};
use crate::syscall;
use crate::time;
use crate::collection::linked_list::*;
use crate::collection::boxed::Box;
use crate::sync::simple_mutex::SimpleMutex;

use bern_arch::{ICore, IScheduler};
use bern_arch::arch::{ArchCore, Arch};

// todo: make these values configurable, proc macro?
pub const TASK_PRIORITIES: usize = 8;
pub const TASK_POOL_SIZE: usize = 16;

type TaskPool = StaticListPool<Task, TASK_POOL_SIZE>;
static TASK_POOL: TaskPool = StaticListPool::new([None; TASK_POOL_SIZE]);

static SCHEDULER: SimpleMutex<Option<Scheduler>> = SimpleMutex::new(None);

pub struct Scheduler
{
    core: ArchCore,
    task_running: Option<Box<Node<Task>>>,
    tasks_ready: [LinkedList<Task, TaskPool>; TASK_PRIORITIES],
    tasks_sleeping: LinkedList<Task, TaskPool>,
    tasks_terminated: LinkedList<Task, TaskPool>,
}


pub fn init() {
    let core = ArchCore::new();

    if let Some(sched) = SCHEDULER.lock() {
        sched.replace(Scheduler {
            core,
            task_running: None,
            tasks_ready: arr![LinkedList::new(&TASK_POOL); 8],
            tasks_sleeping: LinkedList::new(&TASK_POOL),
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

    // ensure an idle task is present
    if sched.tasks_ready[TASK_PRIORITIES-1].len() == 0 {
        Task::new()
            .priority(task::Priority(7))
            .static_stack(crate::alloc_static_stack!(128))
            .spawn(move || default_idle());
    }

    let mut task = None;
    for list in sched.tasks_ready.iter_mut() {
        if list.len() > 0 {
            task = list.pop_front();
            break;
        }
    }
    sched.task_running = task;

    sched.core.start();

    let stack_ptr = sched.task_running.as_ref().unwrap().inner().stack_ptr();
    SCHEDULER.release();

    Arch::start_first_task(stack_ptr);
}

pub fn add(mut task: Task) {
    match SCHEDULER.lock() {
        Some(sched) => {
            unsafe {
                let stack_ptr = Arch::init_task_stack(
                    task.stack_ptr(),
                    task::entry as *const usize,
                    task.runnable_ptr(),
                    syscall::task_exit as *const usize
                );
                task.set_stack_ptr(stack_ptr);
            }
            let prio: usize = task.priority().into();
            sched.as_mut().unwrap().tasks_ready[prio].emplace_back(task).ok();
            SCHEDULER.release();
        },
        None => panic!("Scheduler already locked, (todo reentrant scheduler)"),
    };
}

pub fn sleep(ms: u32) {
    let sched = match SCHEDULER.lock() {
        Some(sched) => sched.as_mut().unwrap(),
        None => panic!("Scheduler already locked, (todo reentrant scheduler)"),
    };

    let task = sched.task_running.as_mut().unwrap().inner_mut();
    task.sleep(ms);
    task.set_transition(Transition::Sleeping);
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
    let mut cursor = sched.tasks_sleeping.cursor_front_mut();
    while let Some(task) = cursor.inner() {
        if task.next_wut() <= now as u64 {
            // todo: this is inefficient, we know that node exists
            if let Some(node) = cursor.take() {
                let prio: usize = node.inner().priority().into();
                sched.tasks_ready[prio].push_back(node);
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

fn default_idle() {
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

////////////////////////////////////////////////////////////////////////////////

/// This function must be called from the architecture specific task switch
/// implementation.
#[no_mangle]
fn switch_context(stack_ptr: u32) -> u32 {
    let mut sched = match SCHEDULER.lock() {
        Some(sched) => sched.as_mut().unwrap(),
        None => panic!("Scheduler already locked, (todo reetrant scheduler)"),
    };
    sched.task_running.as_mut().unwrap().inner_mut().set_stack_ptr(stack_ptr as *mut usize);

    let mut pausing = sched.task_running.take().unwrap();
    let prio: usize = pausing.inner().priority().into();
    match pausing.inner().transition() {
        Transition::None => sched.tasks_ready[prio].push_back(pausing),
        Transition::Sleeping => {
            pausing.inner_mut().set_transition(Transition::None);
            sched.tasks_sleeping.insert_when(pausing, |pausing, task | {
                pausing.next_wut() < task.next_wut()
            });
        },
        Transition::Terminating => {
            pausing.inner_mut().set_transition(Transition::None);
            sched.tasks_terminated.push_back(pausing);
        },
        _ => (),
    }

    // load next task
    let mut task = None;
    for list in sched.tasks_ready.iter_mut() {
        if list.len() > 0 {
            task = list.pop_front();
            break;
        }
    }
    if task.is_none() {
        panic!("Idle task must not be suspended");
    }
    sched.task_running = task;
    let stack_ptr = sched.task_running.as_ref().unwrap().inner().stack_ptr();
    SCHEDULER.release();
    stack_ptr as u32
}