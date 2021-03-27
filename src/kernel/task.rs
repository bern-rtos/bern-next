#[derive(Debug)]
pub struct TaskError;

pub struct Task {
    entry: &'static fn() -> Result<(), TaskError>
}

impl Task {
    //pub fn new(entry: impl FnMut() -> Result<(), TaskError>) -> Task {
    pub fn new(entry: &'static fn() -> Result<(), TaskError>) -> Task {
        Task {
            entry: entry
        }
    }

    pub fn run(&self) -> Result<(), TaskError> {
        let entry = self.entry;
        entry()
    }
}