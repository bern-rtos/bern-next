use crate::memory_protection::IMemoryProtection;
use crate::arch::Arch;

use cortex_m::peripheral::{self, mpu};
use cortex_m::asm;
use cortex_m_rt::exception;

unsafe fn mpu() -> &'static mut mpu::RegisterBlock {
    &mut *(peripheral::MPU::PTR as *mut _)
}

impl IMemoryProtection for Arch {
    fn enable_memory_protection() {
        let mpu =  unsafe{ mpu() };
        unsafe {
            mpu.ctrl.write(5);
        }
        asm::dsb();
        asm::isb();
    }

    fn disable_memory_protection() {
        let mpu =  unsafe{ mpu() };
        asm::dmb();
        unsafe {
            mpu.ctrl.write(0);
        }
    }

    fn protect_memory_region(region: u8, addr: *const usize, size: usize, mode: usize) {
        let mpu =  unsafe{ mpu() };
        let rbar: u32 =
            addr as u32 & !((2 << size) - 1) |   // address
            (1 << 4) |                    // use region number defined below
            region as u32;                // region number

        let rasr: u32 =
            mode as u32 |
            (size as u32) << 1 |
            1; // enable

        unsafe {
            mpu.rbar.write(rbar);
            mpu.rasr.write(rasr);
        }
    }
}

#[allow(non_snake_case)]
#[exception]
fn MemoryManagement() -> () {
    cortex_m::asm::bkpt();
}