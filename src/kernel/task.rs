use super::scheduler;
use super::scheduler::Scheduler;
use super::boxed::Box;
use core::ptr::NonNull;
use core::mem::size_of;

#[derive(Debug)]
pub struct TaskError;

// todo: enforce alignment and size restrictions
// todo: a stack section to memory
#[macro_export]
macro_rules! alloc_static_stack {
    ($size:expr) => {
        {
            static mut STACK: [u8; $size] = [0; $size];
            unsafe { STACK.as_mut() }
        }
    };
}

////////////////////////////////////////////////////////////////////////////////

/// Issue with closures and static tasks
/// ------------------------------------
/// Every closure has its own anonymous type. A closure can only be stored in a
/// generic struct. The task object stored in the task "list" (array) must all
/// have the same size -> not generic. Thus, the closure can only be referenced
/// as trait object. But need to force the closure to be static, so our
/// reference can be as well. A static closure is not possible, as every static
/// needs a specified type.
/// To overcome the issue of storing a closure into a static task we need to
/// wrap the closure in struct and **copy** it into a static stack. Access to
/// the closure is provided via a trait object, which now references a static
/// object which cannot go out of scope.

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

// todo: manage lifetime of stack & runnable
pub struct Task<'a>
{
    runnable: &'a mut (dyn Runnable + 'static),
    next_wut: u64,
    stack_ptr: *const u32, // todo: remove platform dependency
}

impl<'a> Task<'a>
{
    // todo: replace stack with own type
    pub fn spawn<F>(closure: F, stack: &mut [u8])
        where F: 'static + Sync + FnMut() -> Result<(), TaskError>
    {
        let mut runnable = RunnableClosure::new(closure);
        let mut task = Task {
            runnable: Box::new(runnable, stack),
            next_wut: 0,
            stack_ptr: unsafe { stack.as_ptr().offset(size_of::<F>() as isize)} as *const u32,
        };

        Scheduler::add(task);
        // todo: task handle?
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