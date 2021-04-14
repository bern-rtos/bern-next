use core::sync::atomic::{self, Ordering};

use super::st_nucleo_f446::StNucleoF446;
use stm32f4xx_hal::prelude::*;

//use rtt_target::{rtt_init_print, ChannelMode::BlockIfFull};
use bern_test::serial::{self, Serial};
use nb::Error::{WouldBlock, Other};

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut board = StNucleoF446::new();

    let (tx, rx) = board.vcp.split();
    //rtt_init_print!(BlockIfFull);
    bern_test_serial_uplink(tx);
    bern_test_serial_downlink(rx);

    super::super::tests::runner();

    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

/// Current bern test only supports inefficient, blocking serial transfer.
/// A communication interface must implement the embedded-hal serial trait.
fn bern_test_serial_uplink<T>(mut tx: T)
    where T: embedded_hal::serial::Write<u8> + 'static,
{
    Serial::set_write(move |b| {
        match tx.write(b) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                WouldBlock => Err(WouldBlock),
                _ => Err(Other(serial::Error::Peripheral)),
            }
        }
    });
}

fn bern_test_serial_downlink<R>(mut rx: R)
    where R: embedded_hal::serial::Read<u8> + 'static,
{
    Serial::set_read(move || {
        match rx.read() {
            Ok(b) => Ok(b),
            Err(e) => match e {
                WouldBlock => Err(WouldBlock),
                _ => Err(Other(serial::Error::Peripheral)),
            }
        }
    });
}