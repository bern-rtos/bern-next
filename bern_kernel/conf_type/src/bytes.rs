#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct Bytes(usize);

#[allow(non_snake_case)]
impl Bytes {
    pub const fn from_B(bytes: usize) -> Self {
        Bytes(bytes)
    }

    pub const fn from_kB(kilo_bytes: usize) -> Bytes {
        Bytes(kilo_bytes << 10)
    }

    pub const fn from_MB(mega_bytes: usize) -> Bytes {
        Bytes(mega_bytes << 20)
    }

    pub const fn from_GB(giga_bytes: usize) -> Bytes {
        Bytes(giga_bytes << 30)
    }
}

impl Into<usize> for Bytes {
    fn into(self) -> usize {
        self.0
    }
}