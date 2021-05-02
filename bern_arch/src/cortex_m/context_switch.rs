use crate::context_switch::ContextSwitch;
use crate::arch::Arch;
use cortex_m::peripheral::SCB;

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
        "bl    switch_context",
        "pop   {{lr}}",
        "mov   r3, #2",        // todo: read from function
        "msr   control, r3",   // switch to unprivileged thread mode
        "ldmia r0!, {{r4-r11}}",
        "msr   psp, r0",
        "bx    lr",
        options(noreturn),
        )
    }
}

impl ContextSwitch for Arch {
    #[inline]
    fn trigger_context_switch() {
        SCB::set_pendsv();
    }

    fn start_first_task(stack_ptr: *const usize) -> ! {
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
}