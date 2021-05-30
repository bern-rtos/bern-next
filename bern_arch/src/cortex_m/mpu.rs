use cortex_m::peripheral::{self, mpu, MPU};
use cortex_m::asm;

// based on https://github.com/helium/cortex-mpu

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Size {
    S32 = 4,
    S64 = 5,
    S128 = 6,
    S256 = 7,
    S512 = 8,
    S1K = 9,
    S2K = 10,
    S4K = 11,
    S8K = 12,
    S16K = 13,
    S32K = 14,
    S64K = 15,
    S128K = 16,
    S256K = 17,
    S512K = 18,
    S1M = 19,
    S2M = 20,
    S4M = 21,
    S8M = 22,
    S16M = 23,
    S32M = 24,
    S64M = 25,
    S128M = 26,
    S256M = 27,
    S512M = 28,
    S1G = 29,
    S2G = 30,
    S4G = 31,
}
impl Size {
    pub fn bits(self) -> u32 {
        self as u32
    }
}

/*
pub struct Region {
    size: u8,
    base: Size,
    subregions_enabled: u8,
    align: usize,
}

impl Region {
    pub const S32: Self = Self { base: Size::S32, size: 32, subregions_enabled: 8, align: 32 };
    pub const S64: Self = Self { base: Size::S64, size: 64, subregions_enabled: 8, align: 64 };
    pub const S128: Self = Self { base: Size::S128, size: 128, subregions_enabled: 8, align: 128 };

    pub const S96: Self = Self { base: Size::S256, size: 96, subregions_enabled: 1, align: 96 };
    pub const S160: Self = Self { base: Size::S256, size: 224, subregions_enabled: 7, align: 224 };
    pub const S192: Self = Self { base: Size::S256, size: 224, subregions_enabled: 7, align: 224 };
    pub const S224: Self = Self { base: Size::S256, size: 224, subregions_enabled: 7, align: 224 };
    pub const S256: Self = Self { base: Size::S256, size: 256, subregions_enabled: 8, align: 256 };
}
*/

/* Control Register */
const MPU_ENABLE: u32 = 1;
const MPU_HARD_FAULT_ENABLED: u32 = 1 << 1;
const MPU_PRIVILEGED_DEFAULT_ENABLE: u32 = 1 << 2;

/* Region Number Register */
pub enum RegionNumber {
    Ignore,
    Use(u8),
}

/* Region Attribute and Status Register */
const MPU_REGION_ENABLE: u32 = 1;

pub enum Permission {
    NoAccess,
    ReadOnly,
    ReadWrite,
}

impl From<crate::memory_protection::Permission> for Permission {
    fn from(permission: crate::memory_protection::Permission) -> Self {
        match permission {
            crate::memory_protection::Permission::NoAccess => Permission::NoAccess,
            crate::memory_protection::Permission::ReadOnly => Permission::ReadOnly,
            crate::memory_protection::Permission::ReadWrite => Permission::ReadWrite,
        }
    }
}

pub enum Attributes {
    StronglyOrdered,
    Device {
        shareable: bool,
    },
    Normal {
        shareable: bool,
        /// (inner, outer, write allocate)
        cache_policy: (CachePolicy, CachePolicy),
    },
}

pub enum CachePolicy {
    NoCache,
    WriteThrough,
    WriteBack {
        /// write allocate
        wa: bool,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Subregions(u8);
impl Subregions {
    pub const ALL: Subregions = Subregions(0xFF);
    pub const NONE: Subregions = Subregions(0);

    pub fn bits(self) -> u32 {
        !self.0 as u32
    }
}

pub struct Mpu<'a>(&'a mut mpu::RegisterBlock);

impl Mpu<'_> {
    #[inline]
    pub unsafe fn take() -> Self {
        Self(&mut *(peripheral::MPU::PTR as *mut _))
    }

    #[inline]
    pub fn enable(&mut self) {
        unsafe {
            self.0.ctrl.write(MPU_ENABLE | MPU_PRIVILEGED_DEFAULT_ENABLE);
        }
        asm::dsb();
        asm::isb();
    }

    #[inline]
    pub fn disable(&mut self) {
        asm::dmb();
        unsafe {
            self.0.ctrl.write(0);
        }
    }

    #[inline]
    pub fn set_region_base_address(&mut self, addr: u32, region: RegionNumber) {
        let base_addr = addr & !0x1F;
        let (valid, region) = match region {
            RegionNumber::Ignore => (0, 0),
            RegionNumber::Use(region) => (1, region),
        };

        unsafe {
            self.0.rbar.write(base_addr | valid << 4 | region as u32);
        }
    }

    #[inline]
    pub fn set_region_attributes(&mut self,
                                 executable: bool,
                                 access: (Permission, Permission),
                                 attributes: Attributes,
                                 subregions: Subregions,
                                 region_size: Size) {

        // (privileged, unprivileged)
        let ap = match access {
            (Permission::NoAccess, Permission::NoAccess) => 0b000,
            (Permission::ReadWrite, Permission::NoAccess) => 0b001,
            (Permission::ReadWrite, Permission::ReadOnly) => 0b010,
            (Permission::ReadWrite, Permission::ReadWrite) => 0b011,
            (Permission::ReadOnly, Permission::NoAccess) => 0b101,
            (Permission::ReadOnly, Permission::ReadOnly) => 0b111,
            (_, _) => 0b000, // no access
        };

        let (tex, c, b, s) = match attributes {
            Attributes::StronglyOrdered => (0b000, 0, 0, 0),
            Attributes::Device { shareable: true } => (0b000, 0, 1, 0),
            Attributes::Device { shareable: false } => (0b010, 0, 0, 0),
            Attributes::Normal { shareable, cache_policy } => match cache_policy {
                (CachePolicy::WriteThrough, CachePolicy::WriteThrough) => (0b000, 1, 0, shareable as u32),
                (CachePolicy::WriteBack { wa: false }, CachePolicy::WriteBack { wa: false }) => (0b000, 1, 1, shareable as u32),
                (CachePolicy::NoCache, CachePolicy::NoCache) => (0b001, 0, 0, shareable as u32),
                (CachePolicy::WriteBack { wa: true }, CachePolicy::WriteBack { wa: true }) => (0b001, 1, 1, shareable as u32),

                (CachePolicy::NoCache, CachePolicy::WriteBack { wa: true }) => (0b100, 0, 1, shareable as u32),
                (CachePolicy::NoCache, CachePolicy::WriteThrough) => (0b100, 1, 0, shareable as u32),
                (CachePolicy::NoCache, CachePolicy::WriteBack { wa: false }) => (0b100, 1, 1, shareable as u32),
                (CachePolicy::WriteBack { wa: true }, CachePolicy::NoCache) => (0b101, 0, 0, shareable as u32),
                (CachePolicy::WriteBack { wa: true }, CachePolicy::WriteThrough) => (0b101, 1, 0, shareable as u32),
                (CachePolicy::WriteBack { wa: true }, CachePolicy::WriteBack { wa: false }) => (0b101, 1, 1, shareable as u32),
                (CachePolicy::WriteThrough, CachePolicy::NoCache) => (0b110, 0, 0, shareable as u32),
                (CachePolicy::WriteThrough, CachePolicy::WriteBack { wa: true}) => (0b110, 0, 1, shareable as u32),
                (CachePolicy::WriteThrough, CachePolicy::WriteBack { wa: false}) => (0b110, 1, 1, shareable as u32),
                (CachePolicy::WriteBack { wa: false }, CachePolicy::NoCache) => (0b111, 0, 0, shareable as u32),
                (CachePolicy::WriteBack { wa: false }, CachePolicy::WriteBack { wa: true}) => (0b111, 0, 1, shareable as u32),
                (CachePolicy::WriteBack { wa: false }, CachePolicy::WriteThrough) => (0b111, 1, 0, shareable as u32),
            },
        };

        let register = (!executable as u32) << 28 |
            ap << 24 |
            tex << 19 | s << 18 | c << 17 | b << 16 |
            subregions.bits() << 8 |
            region_size.bits() << 1 |
            MPU_REGION_ENABLE;

        unsafe {
            self.0.rasr.write(register);
        }
    }

    #[inline]
    pub fn disable_region(&mut self, region: u8) {
        unsafe {
            self.0.rnr.write(region as u32);
            self.0.rasr.write(0);
        }
    }
}