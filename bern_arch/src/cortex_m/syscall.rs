use crate::arch::Arch;
use crate::syscall::ISyscall;

impl ISyscall for Arch {
    #[inline(always)]
    fn syscall(service: u8, arg0: usize, arg1: usize, arg2: usize) {
        // we need to move the arguments to the correct registers, because the
        // function is inlined
        unsafe { asm!(
            "mov r0, {}",
            "mov r1, {}",
            "mov r2, {}",
            "mov r3, {}",
            "svc 0",
            in(reg) service,
            in(reg) arg0,
            in(reg) arg1,
            in(reg) arg2,
        )}
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
/// The exception link register tells SVC which privilege mode the callee used
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
    //"tst lr, #4",         // check which stack was used
    //"itte eq",
    //"mrseq r4, msp",      // load main stack
    //"addeq r4, #12",      // we pushed r4-r5 + lr
    //"mrsne r4, psp",      // or load process tack
    //"ldr r5, [r4, #24]",  // get callee link register (6 words offset)
    //"ldrb r0, [r5, #-2]", // load the service id from code
    "bl syscall_handler",
    "pop {{r4-r5,lr}}",
    "bx lr",
    options(noreturn),
    );
}