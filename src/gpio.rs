use std::time::{Duration, Instant};
use std::fmt;
use serde::{Serialize, Deserialize};

use anyhow::{Result, Context};
use rppal::gpio::{Gpio, InputPin, Level};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitEvent {
    Open,
    Closed,
}

#[derive(Debug, PartialEq)]
enum FsmState {
    Idle,
    DebouncingHigh,
    High,
    DebouncingLow,
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
    debounce_delay: Duration,
    state: FsmState,
    last_potential_transition_time: Instant,
}

impl GpioMonitor {
    pub fn new(pin_number: u8, poll_interval: Duration, debounce_delay: Duration) -> Result<Self> {
        let gpio = Gpio::new()
            .context("Failed to initialize GPIO")?;
        let pin = gpio.get(pin_number)
            .context(format!("Failed to get GPIO pin {}", pin_number))?
            .into_input_pullup();

        Ok(Self { 
            pin, 
            poll_interval,
            debounce_delay,
            state: FsmState::Idle,
            last_potential_transition_time: Instant::now(),
        })
    }

    pub async fn monitor<F>(&mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(CircuitEvent) + Send + 'static,
    {
        loop {
            let current_switch_reading = self.get_current_state() == CircuitEvent::Closed;
            let time_since_last_change = self.last_potential_transition_time.elapsed();

            match self.state {
                FsmState::Idle => {
                    if current_switch_reading {
                        self.state = FsmState::DebouncingHigh;
                        self.last_potential_transition_time = Instant::now();
                    }
                }
                
                FsmState::DebouncingHigh => {
                    if !current_switch_reading {
                        self.state = FsmState::Idle;
                    } else if time_since_last_change >= self.debounce_delay {
                        self.state = FsmState::High;
                        callback(CircuitEvent::Closed);
                    }
                }
                
                FsmState::High => {
                    if !current_switch_reading {
                        self.state = FsmState::DebouncingLow;
                        self.last_potential_transition_time = Instant::now();
                    }
                }
                
                FsmState::DebouncingLow => {
                    if current_switch_reading {
                        self.state = FsmState::High;
                    } else if time_since_last_change >= self.debounce_delay {
                        self.state = FsmState::Idle;
                        callback(CircuitEvent::Open);
                    }
                }
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
