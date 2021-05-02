pub trait ICore {
    fn new() -> Self;
    fn start(&mut self);
    fn bkpt();
}