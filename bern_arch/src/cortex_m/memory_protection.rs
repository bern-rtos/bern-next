use crate::memory_protection::{IMemoryProtection, Config, Type};
use crate::arch::Arch;
use crate::arch::mpu::{Mpu, RegionNumber, Permission, Subregions, Attributes, CachePolicy};
pub use crate::arch::mpu::Size;

use cortex_m::asm;
use cortex_m_rt::exception;

impl IMemoryProtection for Arch {
    type Size = Size;

    fn enable_memory_protection() {
        let mut mpu =  unsafe{ Mpu::take() };
        mpu.enable();
    }

    fn disable_memory_protection() {
        let mut mpu =  unsafe{ Mpu::take() };
        mpu.disable();
    }

    fn protect_memory_region(region: u8, config: Config<Size>) {
        let mut mpu =  unsafe{ Mpu::take() };

        mpu.set_region_base_address(
            config.addr as u32,
            RegionNumber::Use(region)
        );

        let attributes = match config.memory {
            Type::Ram | Type::Flash => Attributes::Normal {
                shareable: true,
                cache_policy: (CachePolicy::WriteThrough, CachePolicy::WriteThrough),
            },
            Type::Peripheral => Attributes::Device {
                shareable: true
            }
        };


        mpu.set_region_attributes(
            config.executable,
            (Permission::from(config.access.system), Permission::from(config.access.user)),
            attributes,
            Subregions::ALL,
            config.size,
        )


    }
}

#[allow(non_snake_case)]
#[exception]
fn MemoryManagement() -> () {
    cortex_m::asm::bkpt();
}