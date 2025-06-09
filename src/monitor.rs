use std::time::Duration;
use anyhow::{Result, Context};
use crate::StatusEvent;

pub trait Monitor: Send + Sync {
    fn monitor(&mut self, callback: Box<dyn FnMut(StatusEvent) + Send>) -> Result<()>;
    fn get_current_state(&self) -> StatusEvent;
}

pub struct GpioMonitor {
    pin: rppal::gpio::InputPin,
    debounce_delay: Duration,
}

impl GpioMonitor {
    pub fn new(pin_number: u8, debounce_delay: Duration) -> Result<Self> {
        let gpio = rppal::gpio::Gpio::new().context("Failed to initialize GPIO")?;
        let pin = gpio
            .get(pin_number)
            .context(format!("Failed to get GPIO pin {}", pin_number))?
            .into_input_pullup();

        Ok(Self {
            pin,
            debounce_delay,
        })
    }
}

impl Monitor for GpioMonitor {
    fn monitor(&mut self, mut callback: Box<dyn FnMut(StatusEvent) + Send>) -> Result<()> {
        self.pin
            .set_async_interrupt(rppal::gpio::Trigger::Both, Some(self.debounce_delay), move |event| {
                match event.trigger {
                    rppal::gpio::Trigger::RisingEdge => callback(StatusEvent::Closed),
                    rppal::gpio::Trigger::FallingEdge => callback(StatusEvent::Open),
                    _ => (), // Ignore other triggers
                }
            })?;

        Ok(())
    }

    fn get_current_state(&self) -> StatusEvent {
        match self.pin.read() {
            rppal::gpio::Level::Low => StatusEvent::Open,
            rppal::gpio::Level::High => StatusEvent::Closed,
        }
    }
}

pub fn create_monitor(pin_number: u8, debounce_delay: Duration) -> Result<Box<dyn Monitor>> {
    Ok(Box::new(GpioMonitor::new(pin_number, debounce_delay)?))
} 