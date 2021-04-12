
use core::{fmt, mem};
use nb::{block, Error::Other};
use core::fmt::Write;

#[derive(Debug)]
pub enum Error {
    Peripheral,
    NoUplink,
    NoDownlink,
    BufferOverrun,
}

static mut SERIAL: Serial = Serial {
    write: None,
    read: None,
};

pub struct Serial {
    write: Option<&'static mut dyn FnMut(u8) -> nb::Result<(), Error>>,
    read: Option<&'static mut dyn FnMut() -> nb::Result<u8, Error>>,
}

// todo: interrupt driven read and write
impl Serial {
    ///
    /// # Safety
    /// We basically want to create a memory leak/unbounded lifetime, so we can
    /// access a serial write function from anywhere. This is quite unsafe, but
    /// at least `mem::transmute` checks that the buffer has the correct size.
    ///
    /// todo: critical section, reentrancy check
    pub fn set_write<F>(write: F)
        where F: FnMut(u8) -> nb::Result<(), Error> + 'static
    {
        static mut TX: [u8; 4] = [0; 4];
        unsafe {
            TX = mem::transmute(&write);
            let write_ptr = &mut *(TX.as_mut_ptr() as *mut F);
            SERIAL.write = Some(write_ptr);
        }
    }

    ///
    /// # Safety
    /// see [set_write]
    pub fn set_read<F>(read: F)
        where F: FnMut() -> nb::Result<u8, Error> + 'static
    {
        static mut RX: [u8; 4] = [0; 4];
        unsafe {
            RX = mem::transmute(&read);
            let read_ptr = &mut *(RX.as_mut_ptr() as *mut F);
            SERIAL.read = Some(read_ptr);
        }
    }

    pub unsafe fn steal() -> &'static mut Self {
        &mut SERIAL
    }

    pub fn write(&mut self, byte: u8) -> nb::Result<(), Error> {
        match &mut self.write {
            Some(w) => (w)(byte),
            _ => Err(nb::Error::Other(Error::NoUplink)),
        }
    }

    #[doc(hidden)]
    pub fn write_str(s: &str) {
        let ser = unsafe { Serial::steal() };
        ser.write_str(s).ok();
    }

    #[doc(hidden)]
    pub fn write_fmt(arg: fmt::Arguments) {
        let ser = unsafe { Serial::steal() };
        ser.write_fmt(arg).ok();
    }

    pub fn read(&mut self) -> nb::Result<u8, Error> {
        match &mut self.read {
            Some(r) => (r)(),
            _ => Err(Other(Error::NoDownlink)),
        }
    }


    pub fn readln(&mut self, buffer: &mut [u8]) -> nb::Result<usize, Error> {
        if self.read.is_none() {
            return Err(Other(Error::NoDownlink));
        }

        for (i, item) in buffer.iter_mut().enumerate() {
            let c = block!(self.read());
            match c {
                Ok(c) => match c {
                    b'\n' | b'\r' => return Ok(i),
                    b => {
                        *item = b;
                    },
                },
                Err(e) => return Err(Other(e)),
            }
        }
        return Err(Other(Error::BufferOverrun));
    }
}

impl fmt::Write for Serial
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.as_bytes()
            .iter()
            .try_for_each(|c| block!(self.write(*c)))
            .map_err(|_| fmt::Error)
    }
}

// from probe-rs
#[macro_export]
macro_rules! sprintln {
    ($fmt:expr) => {
        $crate::serial::Serial::write_str(concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::serial::Serial::write_fmt(format_args!(concat!($fmt, "\n"), $($arg)*));
    };
}

#[macro_export]
macro_rules! sprint {
    ($fmt:expr) => {
        $crate::serial::Serial::write_str($fmt);
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::serial::Serial::write_fmt(format_args!($fmt, $($arg)*));
    };
}

