use core::mem::size_of;
use core::mem::size_of_val;

pub fn spawn<F>(closure: F)
    where F: 'static + Sync + FnMut() {
    //let size = size_of::<F>();
    let size = size_of_val(&closure);
    unsafe { asm!(
        "mov r1, r4",
        "mov r2, r5",
        "mov r0, 42", // service id
        "svc 0",
        in("r4") &closure as *const _ as usize,
        in("r5") size,
    )};
}

pub fn delay(ms: u32) {
    unsafe { asm!(
        "mov r1, r4",
        "mov r0, 1", // service id
        "svc 0",
        in("r4") ms,
    )};
}