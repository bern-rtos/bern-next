use super::scheduler;
use core::marker::PhantomData;

#[derive(Debug)]
pub struct TaskError;

pub trait Context {
    fn delay(&mut self, ms: u32);
}

// todo: replace with 'trait_alias', as soon as it is stable
impl <F,C> Context for Task<F,C>
    where F: FnMut(&mut C) -> Result<(), TaskError>, C: Context
{
    fn delay(&mut self, ms: u32) {
        self.next_wut = scheduler::get_tick() + ms;
    }
}

pub struct Task<F,C>
    where F: FnMut(&mut C) -> Result<(), TaskError>, C: Context
{
    entry: F,
    next_wut: u32,
    phantom: PhantomData<C>,
}

impl<F,C> Task<F,C>
    where F: FnMut(&mut C) -> Result<(), TaskError>, C: Context
{
    pub fn new(entry: F) -> Task<F,C> {
        Task {
            entry: entry,
            next_wut: 0,
            phantom: PhantomData,
        }
    }

    pub fn run(&mut self) -> Result<(), TaskError> {
        let entry = &mut self.entry;
        entry(self)
    }

    pub fn get_next_wut(&self) -> u32 {
        self.next_wut
    }
}