use core::mem;

use crate::scheduler;
use crate::task::{RunnableResult, TaskBuilder};

use bern_arch::{ISyscall, ICore};
use bern_arch::arch::{Arch, ArchCore};


#[repr(u8)]
pub enum Service {
    SchedulerInit,
    MoveClosureToStack,
    TaskSpawn,
    TaskSleep,
    TaskExit,
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

pub fn task_exit() {
    Arch::syscall(
        Service::TaskExit.service_id(),
        0,
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
            scheduler::sleep(ms);
            0
        },
        Service::TaskExit => {
            scheduler::task_terminate();
            0
        },
        _ => {
            ArchCore::bkpt();
            0
        },
    }
}