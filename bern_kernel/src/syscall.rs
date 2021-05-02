use super::task::Task;
use core::mem;
use crate::scheduler;
use cortex_m::asm;
use bern_arch::{arch::Arch, ISyscall};


#[repr(u8)]
pub enum Service {
    SchedulerInit,
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



pub fn spawn(task: Task) {
    Arch::syscall(
        Service::TaskSpawn.service_id(),
        &task as *const _ as usize,
        0,
        0
    );
}

pub fn sleep(ms: u32) {
    Arch::syscall(
        Service::TaskSleep.service_id(),
        ms as usize,
        0,
        0)
    ;
}

pub fn task_exit() {
    unsafe { asm!(
    "mov r0, 3",
    "svc 0",
    )};
}



#[allow(unused_variables)]
#[no_mangle]
fn syscall_handler(service: Service, arg0: usize, arg1: usize, arg2: usize) {
    match service {
        Service::TaskSpawn => {
            let task: Task = unsafe { mem::transmute(*(arg0 as *const Task)) };
            scheduler::add(task);
        },
        Service::TaskSleep => {
            let ms: u32 = arg0 as u32;
            scheduler::sleep(ms);
        },
        Service::TaskExit => scheduler::task_terminate(),
        _ => asm::bkpt(),
    }
}