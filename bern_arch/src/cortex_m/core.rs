use crate::core::Core;
use cortex_m::Peripherals;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::scb;

pub struct ArchCore {
    peripherals: Peripherals,
}

impl Core for ArchCore {
    fn new() -> Self {
        // NOTE(unsafe): we must be able to take the peripherals or else the
        // system is doomed
        let mut peripherals = unsafe { Peripherals::steal() };
        peripherals.SYST.set_clock_source(SystClkSource::Core);
        // this is configured for the STM32F411 which has a default CPU clock of 48 MHz
        peripherals.SYST.set_reload(48_000);
        peripherals.SYST.clear_current();

        ArchCore {
            peripherals
        }
    }

    fn start(&mut self) {
        self.peripherals.SYST.enable_counter();
        self.peripherals.SYST.enable_interrupt();

        // enable PendSV interrupt on lowest priority
        unsafe {
            self.peripherals.SCB.set_priority(scb::SystemHandler::PendSV, 0xFF);
        }
    }
}