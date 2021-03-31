use super::scheduler;

#[derive(Debug)]
pub struct TaskError;

////////////////////////////////////////////////////////////////////////////////

pub struct Context {
    next_wut: u64,
}

impl Context {
    pub fn delay(&mut self, ms: u32) {
        self.next_wut = scheduler::get_tick() + u64::from(ms);
        scheduler::Scheduler::yield_sched();
    }
}

////////////////////////////////////////////////////////////////////////////////

// todo: remove Sync, it's currently needed to share reference to runnable
pub trait Runnable: Sync {
    fn run(&mut self) -> Result<(), TaskError>;
}

pub struct RunnableClosure<F>
    where F: 'static + Sync + FnMut() -> Result<(), TaskError>,
{
    pub runnable: F,
}
impl<F> RunnableClosure<F>
    where F: 'static + Sync + FnMut() -> Result<(), TaskError>,
{
    pub fn new(runnable: F) -> Self {
        RunnableClosure {
            runnable
        }
    }
}
impl<F> Runnable for RunnableClosure<F>
    where F: 'static + Sync + FnMut() -> Result<(), TaskError>,
{
    fn run(&mut self) -> Result<(), TaskError> {
        (self.runnable)()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Task<'a>
{
    runnable: &'a mut (dyn Runnable + 'static),
    next_wut: u64,
}

impl<'a> Task<'a>
{
    pub fn new(runnable: &'a mut (dyn Runnable + 'static)) -> Self {
        Task {
            runnable,
            next_wut: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), TaskError> {
        self.runnable.run()
    }

    pub fn get_next_wut(&self) -> u64 {
        self.next_wut
    }

    pub fn delay(&mut self, ms: u32) {
        self.next_wut = scheduler::get_tick() + u64::from(ms);
    }
}