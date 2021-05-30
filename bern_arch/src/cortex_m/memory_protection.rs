use crate::memory_protection::{IMemoryProtection, Config, Type};
use crate::arch::Arch;
use crate::arch::mpu::{self, Mpu, RegionNumber, Permission, Subregions, Attributes, CachePolicy};
pub use crate::arch::mpu::{Size, MemoryRegion};

use cortex_m::asm;
use cortex_m_rt::exception;

impl IMemoryProtection for Arch {
    type Size = Size;
    type MemoryRegion = MemoryRegion;

    fn enable_memory_protection() {
        let mut mpu =  unsafe{ Mpu::take() };
        mpu.enable();
    }

    fn disable_memory_protection() {
        let mut mpu =  unsafe{ Mpu::take() };
        mpu.disable();
    }

    fn enable_memory_region(region: u8, config: Config<Size>) {
        let mut mpu =  unsafe{ Mpu::take() };

        let memory_region = Self::prepare_memory_region(region, config);
        mpu.set_region(&memory_region);
    }

    fn disable_memory_region(region: u8) {
        let mut mpu = unsafe { Mpu::take() };
        mpu.disable_region(region);
    }

    fn prepare_memory_region(region: u8, config: Config<Self::Size>) -> Self::MemoryRegion {
        let region_base_address = Mpu::prepare_region_base_address(
            config.addr as u32,
            RegionNumber::Use(region)
        );

        let attributes = match config.memory {
            Type::SramInternal => Attributes::Normal {
                shareable: true,
                cache_policy: (CachePolicy::WriteThrough, CachePolicy::WriteThrough),
            },
            Type::SramExternal => Attributes::Normal {
                shareable: true,
                cache_policy: (CachePolicy::WriteBack { wa: true }, CachePolicy::WriteBack { wa: true }),
            },
            Type::Flash => Attributes::Normal {
                shareable: false,
                cache_policy: (CachePolicy::WriteThrough, CachePolicy::WriteThrough),
            },
            Type::Peripheral => Attributes::Device {
                shareable: true
            }
        };

        let region_attributes = Mpu::prepare_region_attributes(
            config.executable,
            (Permission::from(config.access.system), Permission::from(config.access.user)),
            attributes,
            Subregions::ALL,
            config.size,
        );

        MemoryRegion {
            region_base_address_reg: region_base_address,
            region_attribute_size_reg: region_attributes,
        }
    }

    fn prepare_unused_region(region: u8) -> Self::MemoryRegion {
        MemoryRegion {
            region_base_address_reg: mpu::MPU_REGION_VALID | region as u32,
            region_attribute_size_reg: 0, // disable region
        }
    }

    fn apply_regions(memory_regions: &[MemoryRegion; 3]) {
        let mut mpu = unsafe { Mpu::take() };

        mpu.set_regions(memory_regions);
    }
}

#[allow(non_snake_case)]
#[exception]
fn MemoryManagement() -> () {
    cortex_m::asm::bkpt();
}