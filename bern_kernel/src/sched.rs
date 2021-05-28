// NOTE(unsafe): Switching tasks is protected by critical sections. The compiler
// cannot verify critical section, thus they have to marked as unsafe.
#![allow(unsafe_code)]




/// # Basic Concept
/// Keep interrupt latency as short as possible, move work to PendSV.

pub(crate) mod event;

use core::sync::atomic::{self, Ordering};
use core::mem::MaybeUninit;
use arr_macro::arr;

use event::Event;
use crate::task::{self, Task, Transition};
use crate::syscall;
use crate::time;
use crate::sync::critical_section;
use crate::conf;
use crate::mem::{
    linked_list::*,
    boxed::Box,
    array_pool::ArrayPool,
    pool_allocator,
};

use bern_arch::{ICore, IScheduler, IStartup, IMemoryProtection};
use bern_arch::arch::{ArchCore, Arch};
use core::ptr::NonNull;

type TaskPool = ArrayPool<Node<Task>, { conf::TASK_POOL_SIZE }>;
static TASK_POOL: TaskPool = ArrayPool::new([None; conf::TASK_POOL_SIZE]);
type EventPool = ArrayPool<Node<Event>, { conf::MUTEX_POOL_SIZE }>;
static EVENT_POOL: EventPool = ArrayPool::new(arr![None; 32]);

static mut SCHEDULER: MaybeUninit<Scheduler> = MaybeUninit::uninit();

// todo: split scheduler into kernel and scheduler

pub struct Scheduler {
    core: ArchCore,
    task_running: Option<Box<Node<Task>>>,
    tasks_ready: [LinkedList<Task, TaskPool>; conf::TASK_PRIORITIES],
    tasks_sleeping: LinkedList<Task, TaskPool>,
    tasks_terminated: LinkedList<Task, TaskPool>,
    events: LinkedList<Event, EventPool>,
    event_counter: usize,
}


pub fn init() {
    Arch::init_static_memory();

    /* allow flash read/exec */
    Arch::protect_memory_region(
        0,
        0x08000000 as *const _,
        17, // 512kB
        0b110 << 24 | 1 << 18 | 1 << 17); // RO, 'normal', cacheable

    /* allow peripheral RW */
    Arch::protect_memory_region(
        1,
        0x4000_0000 as *const _,
        28, // 512MB
        0b11 << 24 | 1 << 16); // RW, 'device', no cache

    /* allow .shared section RW access */
    let shared = Arch::region();
    Arch::protect_memory_region(
        1,
        shared.start,
        7, // just guess 256B
        0b11 << 24 | 1 << 18 | 1 << 17); // RW, 'normal', cacheable

    let core = ArchCore::new();

    unsafe {
        SCHEDULER = MaybeUninit::new(Scheduler {
            core,
            task_running: None,
            tasks_ready: arr![LinkedList::new(&TASK_POOL); 8],
            tasks_sleeping: LinkedList::new(&TASK_POOL),
            tasks_terminated: LinkedList::new(&TASK_POOL),
            events: LinkedList::new(&EVENT_POOL),
            event_counter: 0,
        });
    }
}


pub fn start() -> ! {
    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    // ensure an idle task is present
    if sched.tasks_ready[conf::TASK_PRIORITIES-1].len() == 0 {
        Task::new()
            .idle_task()
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
    Arch::start_first_task(stack_ptr);
}

pub fn add(mut task: Task) {
    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable

    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
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
        sched.tasks_ready[prio].emplace_back(task).ok();
    });
}

pub fn sleep(ms: u32) {
    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        let task = sched.task_running.as_mut().unwrap().inner_mut();
        task.sleep(ms);
        task.set_transition(Transition::Sleeping);
    });
    Arch::trigger_context_switch();
}

pub fn yield_now() {
    Arch::trigger_context_switch();
}

pub fn task_terminate() {
    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        let task = sched.task_running.as_mut().unwrap().inner_mut();
        task.set_transition(Transition::Terminating);
    });
    Arch::trigger_context_switch();
}

pub fn tick_update() {
    let now = time::tick();

    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };
    let mut trigger_switch = false;
    critical_section::exec(|| {
        // update pending -> ready list
        let preempt_prio = match sched.task_running.as_ref() {
            Some(task) => task.inner().priority().into(),
            None => usize::MAX,
        };

        let mut cursor = sched.tasks_sleeping.cursor_front_mut();
        while let Some(task) = cursor.inner() {
            if task.next_wut() <= now as u64 {
                // todo: this is inefficient, we know that node exists
                if let Some(node) = cursor.take() {
                    let prio: usize = node.inner().priority().into();
                    sched.tasks_ready[prio].push_back(node);
                    if prio < preempt_prio {
                        trigger_switch = true;
                    }
                }
            } else {
                break; // the list is sorted by wake-up time, we can abort early
            }
            cursor.move_next();
        }

        // time-slicing
        if sched.tasks_ready[preempt_prio].len() > 0 {
            trigger_switch = true;
        }
    });
    if trigger_switch {
        Arch::trigger_context_switch();
    }
}

fn default_idle() {
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

pub fn event_register() -> Result<usize, pool_allocator::Error> {
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        let id = sched.event_counter + 1;
        sched.event_counter = id;
        let result = sched.events.emplace_back(Event::new(id));
        result.map(|_| id)
    })
}

pub fn event_await(id: usize, _timeout: usize) -> Result<(), event::Error> {
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        let event = match sched.events.iter_mut().find(|e|
            e.id() == id
        ) {
            Some(e) => unsafe { NonNull::new_unchecked(e) },
            None => {
                return Err(event::Error::InvalidId);
            }
        };

        let task = sched.task_running.as_mut().unwrap();
        task.inner_mut().set_blocking_event(event);
        task.inner_mut().set_transition(Transition::Blocked);
        return Ok(());
    }).map(|_| Arch::trigger_context_switch())
    // todo: returning ok will not work, because the result will be returned to the wrong task
}

pub fn event_fire(id: usize) {
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };
    let mut switch = false;

    critical_section::exec(|| {
        if let Some(e) = sched.events.iter_mut().find(|e| e.id() == id) {
            if let Some(t) = e.pending.pop_front() {
                let prio: usize = t.inner().priority().into();
                sched.tasks_ready[prio].push_back(t);
            }
            switch = true;
        }
    });

    if switch {
        Arch::trigger_context_switch();
    }
}

////////////////////////////////////////////////////////////////////////////////

/// This function must be called from the architecture specific task switch
/// implementation.
#[no_mangle]
fn switch_context(stack_ptr: u32) -> u32 {
    Arch::disable_memory_protection();
    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    let stack_ptr = critical_section::exec(|| {
        sched.task_running.as_mut().unwrap().inner_mut().set_stack_ptr(stack_ptr as *mut usize);

        let mut pausing = sched.task_running.take().unwrap();
        let prio: usize = pausing.inner().priority().into();
        match pausing.inner().transition() {
            Transition::None => sched.tasks_ready[prio].push_back(pausing),
            Transition::Sleeping => {
                pausing.inner_mut().set_transition(Transition::None);
                sched.tasks_sleeping.insert_when(
                    pausing,
                    |pausing, task| {
                        pausing.next_wut() < task.next_wut()
                    });
            },
            Transition::Blocked => {
                let event = pausing.inner().blocking_event().unwrap(); // cannot be none
                pausing.inner_mut().set_transition(Transition::None);
                unsafe { &mut *event.as_ptr() }.pending.insert_when(
                    pausing,
                    |pausing, task| {
                        pausing.priority().0 < task.priority().0
                    });
            }
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

        Arch::protect_memory_region(
            3,
            task.as_ref().unwrap().inner().stack_top(),
            8,
            0b11 << 24 | 1 << 28 | 1 << 18 | 1 << 17); // RW, no exec, 'normal', cacheable

        sched.task_running = task;
        let stack_ptr = sched.task_running.as_ref().unwrap().inner().stack_ptr();
        stack_ptr as u32
    });

    Arch::enable_memory_protection();
    stack_ptr
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    #[test]
    fn init() {
        Arch::disable_interrupts_context().expect().return_once(|priority| {
            assert_eq!(priority, usize::MAX);
        });
        Arch::enable_interrupts_context().expect().returning(|| {});

        let core_ctx = ArchCore::new_context();
        core_ctx.expect()
            .returning(|| {
                ArchCore::default()
            });

        super::init();

        let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

        critical_section::exec(|| {
            assert_eq!(sched.task_running.is_none(), true);
            assert_eq!(sched.tasks_terminated.len(), 0);
        });
    }
}