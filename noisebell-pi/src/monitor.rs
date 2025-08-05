use std::time::Duration;
use anyhow::Result;
use crate::StatusEvent;

pub trait Monitor: Send + Sync {
    fn monitor(&mut self, callback: Box<dyn FnMut(StatusEvent) + Send>) -> Result<()>;
    fn get_current_state(&self) -> StatusEvent;
}

pub fn create_monitor(monitor_type: &str, pin_number: u8, debounce_delay: Duration, web_port: Option<u16>) -> Result<Box<dyn Monitor>> {
    match monitor_type {
        "gpio" => Ok(Box::new(crate::gpio_monitor::GpioMonitor::new(pin_number, debounce_delay)?)),
        "web" => {
            let port = web_port.ok_or_else(|| anyhow::anyhow!("Web monitor requires a port number"))?;
            Ok(Box::new(crate::web_monitor::WebMonitor::new(port)?))
        },
        _ => Err(anyhow::anyhow!("Unknown monitor type: {}", monitor_type)),
    }
} 