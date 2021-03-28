use super::scheduler;

#[derive(Debug)]
pub struct TaskError;

pub struct Context {
    next_wut: u64,
}

impl Context {
    pub fn delay(&mut self, ms: u32) {
        self.next_wut = scheduler::get_tick() + u64::from(ms);

    }
}

// todo: replace with 'trait_alias', as soon as it is stable
pub struct Task<F>
    where F: FnMut(&mut Context) -> Result<(), TaskError>
{
    entry: F,
    context: Context,
}

impl<F> Task<F>
    where F: FnMut(&mut Context) -> Result<(), TaskError>
{
    pub fn new(entry: F) -> Task<F> {
        Task {
            entry: entry,
            context: Context{
                next_wut: 0,
            }
        }
    }

    pub fn run(&mut self) -> Result<(), TaskError> {
        let entry = &mut self.entry;
        entry(&mut self.context)
    }

    pub fn get_next_wut(&self) -> u64 {
        self.context.next_wut
    }
}