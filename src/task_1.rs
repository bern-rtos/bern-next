use crate::kernel::task_trait::TaskTrait;

pub struct Task1<Led> {
    led: Led,
}

impl<Led> Task1<Led>
    where Led: embedded_hal::digital::v2::ToggleableOutputPin
{
    pub fn new(led: Led) -> Task1<Led> {
        Task1 {
            led: led,
        }
    }

    fn toggle(&mut self) {
        self.led.toggle();
    }
}

impl<Led> TaskTrait for Task1<Led>
    where Led: embedded_hal::digital::v2::ToggleableOutputPin
{
    fn run(&mut self) -> Result<(), ()> {
        self.toggle();
        Ok(())
    }
}