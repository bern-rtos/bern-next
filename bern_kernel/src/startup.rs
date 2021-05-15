use r0;

pub fn init_static() {
    init_shared();
}

fn init_shared() {
    extern "C" {
        static mut __sshared: u32;
        static mut __eshared: u32;
        static __sishared: u32;
    }
    unsafe {
        r0::init_data(&mut __sshared, &mut __eshared, &__sishared);
    }
}