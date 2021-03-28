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
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait Runnable {
    fn run(&mut self, context: &mut Context) -> Result<(), TaskError>;
}

pub struct RunnableClosure<F>
    where F: FnMut(&mut Context) -> Result<(), TaskError>,
{
    runnable: F,
}
impl<F> RunnableClosure<F>
    where F: FnMut(&mut Context) -> Result<(), TaskError>,
{
    pub fn new(runnable: F) -> Self {
        RunnableClosure {
            runnable
        }
    }
}
impl<F> Runnable for RunnableClosure<F>
    where F: FnMut(&mut Context) -> Result<(), TaskError>,
{
    fn run(&mut self, context: &mut Context) -> Result<(), TaskError> {
        (self.runnable)(context)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Task<'a>
{
    pub runnable: &'a mut dyn Runnable,
    context: Context,
}

impl<'a> Task<'a>
{
    pub fn new(runnable: &'a mut dyn Runnable) -> Self {
        Task {
            runnable,
            context: Context {
                next_wut: 0,
            }
        }
    }

    // pub fn new_from_closure<F>(runnable: F) -> (RunnableClosure<F>, Self)
    //     where F: 'static + FnMut(&mut Context) -> Result<(), TaskError>,
    // {
    //     let mut runnable = RunnableClosure::new(runnable);
    //     let task = Self::new(&mut runnable);
    //     return (runnable, task);
    // }

    pub fn run(&mut self) -> Result<(), TaskError> {
        self.runnable.run(&mut self.context)
    }

    pub fn get_next_wut(&self) -> u64 {
        self.context.next_wut
    }
}