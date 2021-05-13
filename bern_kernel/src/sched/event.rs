use crate::mem::linked_list::LinkedList;
use crate::task::Task;

#[repr(u8)]
pub enum Error {
    TimeOut,
    InvalidId,
}

pub enum Wake {
    WakeFirst,
    WakeAll,
}

pub struct Event {
    /// Event identifier (randomize to protect access)
    id: usize,
    /// Tasks waiting for the event
    pub pending: LinkedList<Task, super::TaskPool>,
    /// Wake strategy on event
    wake: Wake,
    /// Apply priority inversion
    priority_inversion: bool,
}

impl Event {
    pub fn new(id: usize) -> Self {
        Event {
            id,
            pending: LinkedList::new(&super::TASK_POOL),
            wake: Wake::WakeFirst,
            priority_inversion: false,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}