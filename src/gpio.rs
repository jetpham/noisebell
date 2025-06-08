use std::time::Duration;
use std::fmt;
use serde::{Serialize, Deserialize};

use anyhow::{Result, Context};
use rppal::gpio::{Gpio, InputPin, Level, Trigger};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitEvent {
    Open,
    Closed,
}

impl fmt::Display for CircuitEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CircuitEvent::Open => write!(f, "open"),
            CircuitEvent::Closed => write!(f, "closed"),
        }
    }
}

pub struct GpioMonitor {
    pin: InputPin,
    debounce_delay: Duration,
}

impl GpioMonitor {
    pub fn new(pin_number: u8, debounce_delay: Duration) -> Result<Self> {
        let gpio = Gpio::new()
            .context("Failed to initialize GPIO")?;
        let pin = gpio.get(pin_number)
            .context(format!("Failed to get GPIO pin {}", pin_number))?
            .into_input_pullup();

        Ok(Self { 
            pin, 
            debounce_delay,
        })
    }

    pub fn monitor<F>(&mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(CircuitEvent) + Send + 'static,
    {
        self.pin.set_async_interrupt(
            Trigger::Both,
            Some(self.debounce_delay),
            move |event| {
                match event.trigger {
                    Trigger::RisingEdge => callback(CircuitEvent::Closed),
                    Trigger::FallingEdge => callback(CircuitEvent::Open),
                    _ => (), // Ignore other triggers
                }
            },
        )?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_current_state(&self) -> CircuitEvent {
        match self.pin.read() {
            Level::Low => CircuitEvent::Open,
            Level::High => CircuitEvent::Closed,
        }
    }
}
