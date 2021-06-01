#![no_std]

pub struct Task {
    pub pool_size: usize,
    pub priorities: u8,
}

pub struct Event {
    pub pool_size: usize,
}

pub struct Conf {
    pub task: Task,
    pub event: Event,
}