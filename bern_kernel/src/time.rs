use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::peripheral::SCB;
use cortex_m_rt::exception;
use crate::scheduler::Scheduler;

static COUNT: AtomicU32 = AtomicU32::new(0); // todo: replace with u64

#[inline(always)]
fn system_tick_update() {
    COUNT.fetch_add(1, Ordering::Relaxed);
    Scheduler::tick_update();
}

#[exception]
fn SysTick() {
    system_tick_update();
}

pub fn tick() -> u64 {
    COUNT.load(Ordering::Relaxed) as u64
}