pub trait ISyscall {
    fn syscall(service: u8, arg0: usize, arg1: usize, arg2: usize) -> usize;
}