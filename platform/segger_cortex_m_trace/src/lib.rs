#![no_std]

use stm32f4xx_hal as hal;
use hal::prelude::*;
use hal::stm32::{
    Peripherals,
    USART2,
};
use hal::gpio::{
    gpioa::PA,
    gpiob::PB,
    gpioc::PC,
    *,
};
use hal::serial::{
    Serial,
    Tx,
    Rx,
};

// todo: can we replace this type madness with a macro?

pub struct SeggerCortexMTrace {
}

impl SeggerCortexMTrace {
    pub fn new() -> Self {
        let stm32_peripherals = Peripherals::take()
            .expect("cannot take stm32 peripherals");

        /* system clock */
        let rcc = stm32_peripherals.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        /* gpio's */
        let gpioa = stm32_peripherals.GPIOA.split();
        let gpiob = stm32_peripherals.GPIOB.split();
        let gpioc = stm32_peripherals.GPIOC.split();


        /* assemble... */
        SeggerCortexMTrace {

        }
    }
}
