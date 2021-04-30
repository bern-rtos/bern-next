use core::mem::size_of;
use core::mem::size_of_val;
use super::task::Task;
use core::mem;
use crate::scheduler::Scheduler;
use cortex_m::asm;
use crate::api::syscall::Syscall;

#[repr(u8)]
pub enum Service {
    KernelInit = 0,
    TaskSpawn = 1,
    TaskDelay = 2,
    TaskExit = 3,
}
impl Service {
    /// Get syscall service id
    pub const fn service_id(self) -> u8 {
        self as u8
    }
}

pub struct ArmCortexM;

impl Syscall for ArmCortexM {
    fn spawn(task: Task) {
        unsafe {
            asm!(
            "mov r1, r4",
            "svc {number}",
            number = const Service::TaskSpawn.service_id(),
            in("r4") &task as *const _ as usize,
            )
        };
    }

    fn sleep(ms: u32) {
        unsafe {
            asm!(
            "mov r1, r4",
            "svc {number}",
            number = const Service::TaskDelay.service_id(),
            in("r4") ms,
            )
        };
    }

    fn task_exit() {
        unsafe { asm!(
        "svc {number}",
        number = const Service::TaskExit.service_id(),
        )};
    }
}

#[no_mangle]
fn syscall_handler(service: Service, arg0: u32, arg1: u32) {
    match service {
        Service::TaskSpawn => {
            let task: Task = unsafe { mem::transmute_copy(&*(arg0 as *const Task)) };
            Scheduler::add(task);
        },
        Service::TaskDelay => Scheduler::sleep(arg0),
        Service::TaskExit => Scheduler::task_terminate(),
        _ => asm::bkpt(),
    }
}


/// Extract and prepare system call for Rust handler.
/// r0 is used to store the service id, r1-r3 can contain call parameter.
///
/// The system call service id (`svc xy`) is not passed on, we have to
/// retrieve it from code memory. Thus we load the stack pointer from the
/// callee and read the link register. The link register is pointing to
/// instruction just after the system call, the system call service id is
/// placed two bytes before that.
///
/// The exception link register tells SVC which privlege mode the calle used
/// | EXC_RETURN (lr) | Privilege Mode     | Stack |
/// |-----------------|--------------------|------ |
/// | 0xFFFFFFF1      | Handler Mode       | MSP   |
/// | 0xFFFFFFF9      | Thread Mode        | MSP   |
/// | 0xFFFFFFFD      | Thread Mode        | PSP   |
/// | 0xFFFFFFE1      | Handler Mode (FPU) | MSP   |
/// | 0xFFFFFFE9      | Thread Mode (FPU)  | MSP   |
/// | 0xFFFFFFED      | Thread Mode (FPU)  | PSP   |
#[no_mangle]
#[naked]
unsafe extern "C" fn SVCall() {
    asm!(
        "push {{r4-r5,lr}}",
        "tst lr, #4",         // check which stack was used
        "itte eq",
        "mrseq r4, msp",      // load main stack
        "addeq r4, #12",      // we pushed r4-r5 + lr
        "mrsne r4, psp",      // or load process tack
        "ldr r5, [r4, #24]",  // get callee link register (6 words offset)
        "ldrb r0, [r5, #-2]", // load the service id from code
        "bl syscall_handler",
        "pop {{r4-r5,lr}}",
        "bx lr",
        options(noreturn),
    );
}