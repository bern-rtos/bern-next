use core::sync::atomic::{self, Ordering};

use stm32f4xx_hal as hal;
use hal::{prelude::*, stm32};

use bern_test::serial::{self, Serial};
use nb::Error::{WouldBlock, Other};

#[cortex_m_rt::entry]
fn main() -> ! {
    /* init board */
    let stm32_peripherals = stm32::Peripherals::take().expect("cannot take stm32 peripherals");

    // Set up the system clock. We want to run at 48MHz for this one.
    let rcc = stm32_peripherals.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

    // gpio's
    let gpioa = stm32_peripherals.GPIOA.split();

    // uart
    let txd = gpioa.pa2.into_alternate_af7();
    let rxd = gpioa.pa3.into_alternate_af7();
    let serial = hal::serial::Serial::usart2(
        stm32_peripherals.USART2,
        (txd, rxd),
        hal::serial::config::Config::default().baudrate(115_200.bps()),
        clocks
    ).unwrap();

    /* bidirectional serial port needed for test framework */
    let (mut tx, mut rx) = serial.split();

    Serial::set_write(move |b| {
        match tx.write(b) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                WouldBlock => Err(WouldBlock),
                _ => Err(Other(serial::Error::Peripheral)),
            }
        }
    });

    Serial::set_read(move || {
        match rx.read() {
            Ok(b) => Ok(b),
            Err(e) => match e {
                WouldBlock => Err(WouldBlock),
                _ => Err(Other(serial::Error::Peripheral)),
            }
        }
    });

    super::super::tests::runner();

    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}