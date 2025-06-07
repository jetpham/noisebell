use std::time::Duration;
use std::fmt;
use serde::{Serialize, Deserialize};

use anyhow::{Result, Context};
use rppal::gpio::{Gpio, InputPin, Level};

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
    poll_interval: Duration,
}

impl GpioMonitor {
    pub fn new(pin_number: u8, poll_interval: Duration) -> Result<Self> {
        let gpio = Gpio::new()
            .context("Failed to initialize GPIO")?;
        let pin = gpio.get(pin_number)
            .context(format!("Failed to get GPIO pin {}", pin_number))?
            .into_input_pullup();

        Ok(Self { pin, poll_interval })
    }

    pub async fn monitor<F>(&mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(CircuitEvent) + Send + 'static,
    {
        let mut previous_state = self.get_current_state();
        callback(previous_state); // Send initial state

        loop {
            let current_state = self.get_current_state();

            if current_state != previous_state {
                callback(current_state);
                previous_state = current_state;
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }

    pub fn get_current_state(&self) -> CircuitEvent {
        match self.pin.read() {
            Level::Low => CircuitEvent::Open,
            Level::High => CircuitEvent::Closed,
        }
    }
}
