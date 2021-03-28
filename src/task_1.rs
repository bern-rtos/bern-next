use crate::kernel::task_trait::TaskTrait;

pub struct Task1 {
    hello: u32,
}

impl Task1 {
    pub fn new() -> Task1 {
        Task1 {
            hello: 1,
        }
    }
}

impl TaskTrait for Task1 {
    fn init(&mut self) {
        unimplemented!()
    }

    fn run(&mut self) {
        unimplemented!()
    }
}