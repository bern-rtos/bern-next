use crate::IStartup;
use crate::arch::Arch;
use r0;
use crate::startup::Region;

extern "C" {
    static mut __sshared: u32;
    static mut __eshared: u32;
    static __sishared: u32;
}

impl IStartup for Arch {
    fn init_static_memory() {
        unsafe {
            let mut shared_ptr = __sshared;
            r0::init_data(&mut shared_ptr, &mut __eshared, &__sishared);
        }
    }

    fn region() -> Region {
        unsafe {
            Region {
                start: __sshared as *const _,
                stop: __eshared as *const _,
            }
        }
    }
}