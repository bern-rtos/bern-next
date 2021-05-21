use crate::IStartup;
use crate::arch::Arch;
use r0;

impl IStartup for Arch {
    fn init_static_memory() {
        extern "C" {
            static mut __sshared: u32;
            static mut __eshared: u32;
            static __sishared: u32;
        }
        unsafe {
            r0::init_data(&mut __sshared, &mut __eshared, &__sishared);
        }
    }
}