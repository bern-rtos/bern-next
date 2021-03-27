pub trait TaskTrait {
    fn run(&mut self) -> Result<(), ()>;
}