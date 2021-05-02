use core::sync::atomic::{AtomicU32, Ordering};
use crate::scheduler;

static COUNT: AtomicU32 = AtomicU32::new(0); // todo: replace with u64

#[no_mangle]
#[inline(always)]
fn system_tick_update() {
    COUNT.fetch_add(1, Ordering::Relaxed);
    scheduler::tick_update();
}

pub fn tick() -> u64 {
    COUNT.load(Ordering::Relaxed) as u64
}