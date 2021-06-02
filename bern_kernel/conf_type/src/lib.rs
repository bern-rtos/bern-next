#![no_std]

pub mod bytes;

pub use bern_arch::arch::memory_protection::Size;

pub struct Task {
    pub pool_size: usize,
    pub priorities: u8,
}

pub struct Event {
    pub pool_size: usize,
}

pub struct MemorySection {
    pub start_address: usize,
    pub size: Size,
}

pub struct Memory {
    pub flash: MemorySection,
    pub sram: MemorySection,
    pub peripheral: MemorySection,
}

pub struct Conf {
    pub task: Task,
    pub event: Event,
    pub memory: Memory,
}