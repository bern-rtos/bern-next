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


pub struct Vcp {
    pub tx: Tx<USART2>,
    pub rx: Rx<USART2>,
}

pub struct ShieldBfh {
    //pub buttons: [PC<Input<Floating>>;8],
    pub led_0: PB<Output<PushPull>>,
    pub led_1: PB<Output<PushPull>>,
    pub led_2: PC<Output<PushPull>>,
    pub led_3: PC<Output<PushPull>>,
    //pub led_4: PA<Output<PushPull>>, // conflict with USART2
    //pub led_5: PA<Output<PushPull>>, // conflict with USART2
    pub led_6: PC<Output<PushPull>>,
    pub led_7: PC<Output<PushPull>>,
}

pub struct StNucleoF446 {
    pub led: Option<PA<Output<PushPull>>>,
    pub button: PC<Input<Floating>>,
    pub vcp: Option<Vcp>, // allow taking vcp and passing the board on, not optimal
    pub shield: ShieldBfh,
}

impl StNucleoF446 {
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

        /* Virtual Com Port (VCP) over debug adapter */
        let txd = gpioa.pa2.into_alternate_af7();
        let rxd = gpioa.pa3.into_alternate_af7();
        let vcp = Serial::usart2(
            stm32_peripherals.USART2,
            (txd, rxd),
            hal::serial::config::Config::default().baudrate(115_200.bps()),
            clocks
        ).unwrap();
        let (vcp_tx, vcp_rx) = vcp.split();

        /* board IOs */
        let led = gpioa.pa5.into_push_pull_output().downgrade();
        let button = gpioc.pc13.into_floating_input().downgrade();

        /* BFH BTE5056 shield */
        let shield_led_0 = gpiob.pb11.into_push_pull_output().downgrade();
        let shield_led_1 = gpiob.pb12.into_push_pull_output().downgrade();
        let shield_led_2 = gpioc.pc2.into_push_pull_output().downgrade();
        let shield_led_3 = gpioc.pc3.into_push_pull_output().downgrade();
        //let shield_led_4 = gpioa.pa2.into_push_pull_output().downgrade();
        //let shield_led_5 = gpioa.pa3.into_push_pull_output().downgrade();
        let shield_led_6 = gpioc.pc6.into_push_pull_output().downgrade();
        let shield_led_7 = gpioc.pc7.into_push_pull_output().downgrade();

        /* assemble... */
        StNucleoF446 {
            led: Some(led),
            button,
            vcp: Some(Vcp {
                tx: vcp_tx,
                rx: vcp_rx,
            }),
            shield: ShieldBfh {
                led_0: shield_led_0,
                led_1: shield_led_1,
                led_2: shield_led_2,
                led_3: shield_led_3,
                //led_4: shield_led_4,
                //led_5: shield_led_5,
                led_6: shield_led_6,
                led_7: shield_led_7,
            },
        }
    }
}
