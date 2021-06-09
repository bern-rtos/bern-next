use core::mem;

use crate::sched;
use crate::task::{RunnableResult, TaskBuilder};

use bern_arch::ISyscall;
use bern_arch::arch::Arch;
use crate::sched::event;


// todo: create with proc macro

#[repr(u8)]
enum Service {
    MoveClosureToStack,
    TaskSpawn,
    TaskSleep,
    TaskYield,
    TaskExit,
    EventRegister,
    EventAwait,
    EventFire,
}

impl Service {
    /// Get syscall service id
    pub const fn service_id(self) -> u8 {
        self as u8
    }
}


pub fn move_closure_to_stack<F>(closure: F, builder: &mut TaskBuilder)
    where F: 'static + Sync + FnMut() -> RunnableResult
{
    Arch::syscall(
        Service::MoveClosureToStack.service_id(),
        &closure as *const _ as usize,
        mem::size_of::<F>() as usize,
        builder as *mut _ as usize
    );
}

pub fn task_spawn(builder: &mut TaskBuilder, runnable: &&mut (dyn FnMut() -> RunnableResult)) {
    Arch::syscall(
        Service::TaskSpawn.service_id(),
        builder as *mut _ as usize,
        runnable as *const _ as usize,
        0
    );
}

pub fn sleep(ms: u32) {
    Arch::syscall(
        Service::TaskSleep.service_id(),
        ms as usize,
        0,
        0
    );
}

pub fn yield_now() {
    Arch::syscall(
        Service::TaskYield.service_id(),
        0,
        0,
        0
    );
}

pub fn task_exit() {
    Arch::syscall(
        Service::TaskExit.service_id(),
        0,
        0,
        0
    );
}

pub fn event_register() -> usize {
    Arch::syscall(
        Service::EventRegister.service_id(),
        0,
        0,
        0
    )
}

pub fn event_await(id: usize, timeout: u32) -> Result<(), event::Error> {
    let ret_code = Arch::syscall(
        Service::EventAwait.service_id(),
        id,
        timeout as usize,
        0
    ) as u8;
    unsafe { mem::transmute(ret_code) }
}

pub fn event_fire(id: usize) {
    Arch::syscall(
        Service::EventFire.service_id(),
        id,
        0,
        0
    );
}


// userland barrier ////////////////////////////////////////////////////////////

// todo: return result
#[allow(unused_variables)]
#[no_mangle]
fn syscall_handler(service: Service, arg0: usize, arg1: usize, arg2: usize) -> usize {
    match service {
        Service::MoveClosureToStack => {
            let builder: &mut TaskBuilder = unsafe { mem::transmute(arg2 as *mut TaskBuilder) };
            TaskBuilder::move_closure_to_stack(
                builder,
                arg0 as *const u8,
                arg1
            );
            0
        },
        Service::TaskSpawn => {
            let builder: &mut TaskBuilder = unsafe { mem::transmute(arg0 as *mut TaskBuilder) };
            let runnable: &&mut (dyn FnMut() -> RunnableResult) = unsafe {
                mem::transmute(arg1 as *mut &mut (dyn FnMut() -> RunnableResult))
            };
            TaskBuilder::build(
                builder,
                runnable,
            );
            0
        },
        Service::TaskSleep => {
            let ms: u32 = arg0 as u32;
            sched::sleep(ms);
            0
        },
        Service::TaskYield => {
            sched::yield_now();
            0
        },
        Service::TaskExit => {
            sched::task_terminate();
            0
        },

        Service::EventRegister => {
            match sched::event_register() {
                Ok(id) => id,
                Err(_) => 0,
            }
        },
        Service::EventAwait => {
            let id = arg0;
            let timeout = arg1;
            let result = sched::event_await(id, timeout);
            let result: Result<(), event::Error> = Ok(());
            let ret_code: u8 = unsafe { mem::transmute(result) };
            ret_code as usize
        },
        Service::EventFire => {
            let id = arg0;
            sched::event_fire(id);
            0
        },
    }
}