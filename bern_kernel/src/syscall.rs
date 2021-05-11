use core::mem;

use crate::scheduler;
use crate::task::{RunnableResult, TaskBuilder};
use crate::sync::mutex;

use bern_arch::{ISyscall, ICore};
use bern_arch::arch::{Arch, ArchCore};
use core::ops::{DerefMut};
use crate::collection::boxed::Box;
use core::ptr::NonNull;
use crate::sync::mutex::MutexInternal;


// todo: create with proc macro

#[repr(u8)]
enum Service {
    SchedulerInit,
    MoveClosureToStack,
    TaskSpawn,
    TaskSleep,
    TaskExit,
    MutexNew,
    MutexLock,
    MutexRelease,
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

pub fn mutex_new<T>(element: &T) -> usize {
    Arch::syscall(
        Service::MutexNew.service_id(),
        element as *const _ as usize,
        0,
        0
    )
}

pub fn mutex_lock(id: usize) -> Result<*mut usize, mutex::Error> {
    match Arch::syscall(
        Service::MutexLock.service_id(),
        id,
        0,
        0
    ) {
        0 => Err(mutex::Error::WouldBlock),
        inner => Ok(inner as *mut usize)
    }
}

pub fn mutex_release(id: usize) {
    Arch::syscall(
        Service::MutexRelease.service_id(),
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
            scheduler::sleep(ms);
            0
        },
        Service::TaskExit => {
            scheduler::task_terminate();
            0
        },
        Service::MutexNew => {
            let element = arg0 as *mut _;
            let mutex = scheduler::mutex_add(mutex::MutexInternal::new(element));
            match mutex {
                Ok(mut m) => m.deref_mut() as *mut _ as usize,
                Err(_) => panic!("could not create mutex"),
            }
        },
        Service::MutexLock => {
            Box::<MutexInternal>::from_raw(NonNull::new(arg0 as *mut _).unwrap())
                .deref_mut()
                .try_lock() as usize
        },
        Service::MutexRelease => {
            Box::<MutexInternal>::from_raw(NonNull::new(arg0 as *mut _).unwrap())
                .deref_mut()
                .release();
            0
        },
        _ => {
            ArchCore::bkpt();
            0
        },
    }
}