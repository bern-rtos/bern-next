pub trait ISync {
    fn disable_interrupts(priority: usize);
    fn enable_interrupts();
}