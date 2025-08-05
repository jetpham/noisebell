# <img src="media/noisebell%20logo.svg" width="100" alt="Noisebell Logo" style="vertical-align: middle; margin-right: 20px;"> Noisebell

A switch monitoring system that detects circuit state changes via GPIO and notifies configured HTTP endpoints via POST requests.

This is build by [Jet Pham][jetpham] to be used at Noisebridge to replace their old discord status bot

## Features

- GPIO circuit monitoring with configurable pin
- HTTP endpoint notifications via POST requests
- Daily rotating log files
- Cross-compilation support for Raspberry Pi deployment
- Software debouncing to prevent noisy switch detection
- Concurrent HTTP notifications for improved performance
- Comprehensive logging and error reporting
- Web-based monitor for testing (no physical hardware required)
- **Unified configuration system** with environment variable support

## Configuration

Noisebell uses a unified configuration system that can be customized through environment variables. Copy `config.example.env` to `.env` and modify the values as needed:

```bash
cp config.example.env .env
```

### Configuration Options

#### GPIO Configuration

- `GPIO_PIN` (default: 17) - GPIO pin number to monitor
- `DEBOUNCE_DELAY_SECS` (default: 5) - Debounce delay in seconds

#### Web Monitor Configuration

- `WEB_MONITOR_PORT` (default: 8080) - Port for the web monitor UI
- `WEB_MONITOR_ENABLED` (default: false) - Enable/disable web monitor

#### Logging Configuration

- `LOG_LEVEL` (default: info) - Log level (trace, debug, info, warn, error)
- `LOG_FILE_PATH` (default: logs/noisebell.log) - Log file path
- `LOG_MAX_FILES` (default: 7) - Maximum number of log files to keep
- `LOG_MAX_FILE_SIZE_MB` (default: 10) - Maximum log file size in MB

#### Monitor Configuration

- `MONITOR_TYPE` (default: gpio) - Monitor type (gpio, web)
- `HEALTH_CHECK_INTERVAL_SECS` (default: 30) - Health check interval

#### Endpoint Configuration

- `ENDPOINT_CONFIG_FILE` (default: endpoints.json) - Path to JSON file containing endpoint configurations

### Example Configuration

```bash
# For development with web monitor
MONITOR_TYPE=web
WEB_MONITOR_ENABLED=true
LOG_LEVEL=debug
ENDPOINT_CONFIG_FILE=endpoints.json

# For production with GPIO
MONITOR_TYPE=gpio
GPIO_PIN=17
LOG_LEVEL=info
ENDPOINT_CONFIG_FILE=endpoints.json
```

### GPIO and Physical Tech

We interact directly over a [GPIO pin in a pull-up configuration][gpio-pullup] to read whether a circuit has been closed with a switch. This is an extremely simple circuit that will internally call a callback function when the state of the circuit changes.

When a state change is detected, the system:

1. Logs the circuit state change
2. Sends HTTP POST requests to all configured endpoints
3. Reports success/failure statistics in the logs

## Debouncing

When a switch changes state, it can bounce and create multiple rapid signals. Debouncing adds a delay to wait for the signal to settle, ensuring we only detect one clean state change instead of multiple false ones.

We do debouncing with software via [`set_async_interupt`][rppal-docs] which handles software debounce for us.

### Logging

Logs are stored in the `logs` directory with daily rotation for the past 7 days (configurable via `LOG_MAX_FILES`)

### Endpoint Notifications

When a circuit state change is detected, the system sends HTTP POST requests to all configured endpoints with the following JSON payload:

```json
{
  "event": "open",
  "timestamp": "2025-01-11T10:30:00Z",
}
```

#### Endpoint Configuration

Endpoints are configured in a JSON file (default: `endpoints.json`) with the following structure:

```json
{
  "endpoints": [
    {
      "url": "https://noisebell.jetpham.com/api/status",
      "name": "Noisebell Website",
      "timeout_secs": 30,
      "retry_attempts": 3
    }  
  ]
} 
```

**Endpoint Configuration Options:**

- `url` (required) - The HTTP endpoint URL to POST to
- `name` (optional) - A friendly name for the endpoint (used in logs)
- `timeout_secs` (optional, default: 30) - Request timeout in seconds
- `retry_attempts` (optional, default: 3) - Number of retry attempts on failure

### Web Monitor

A web-based monitor is available for testing without physical hardware. When `WEB_MONITOR_ENABLED=true`, you can access the monitor at `http://localhost:8080` to manually trigger state changes and test the endpoint notification system.

### Images

<div align="center">
<img src="media/noisebell%20knifeswitch.jpg" width="400" alt="Knife Switch">
<br>
<em>The knife switch used to detect circuit state changes</em>
</div>

<br>

<div align="center">
<img src="media/noisebell%20raspberrypi%20closeup.jpg" width="400" alt="Raspberry Pi Closeup">
<br>
<em>Closeup view of the Raspberry Pi setup</em>
</div>

<br>

<div align="center">
<img src="media/noisebell%20raspberrypi%20with%20porthole.jpg" width="400" alt="Raspberry Pi with Porthole">
<br>
<em>The complete setup showing the Raspberry Pi mounted in a porthole</em>
</div>

## Development

### Requirements

- Rust toolchain (Install [Rust][rust-install])
- Raspberry Pi (tested on [RP02W][rp02w])
- `cross` for cross-compilation (Install [Cross][cross-install])
- Internet connectivity (wifi for the rp02w)

### Local Development (Web Monitor)

For local development and testing, you can run the web-based monitor using the following command:

```bash
MONITOR_TYPE=web WEB_MONITOR_ENABLED=true cargo run
```

Or set up a `.env` file:

```bash
cp config.example.env .env
# Edit .env to set MONITOR_TYPE=web and WEB_MONITOR_ENABLED=true
cargo run
```

This will start a web server on port 8080. Open your browser and go to [http://localhost:8080](http://localhost:8080) to interact with the web monitor.

This is meant to replace the need for tesing on an actual raspberry pi with gpio pins while keeping the terminal clean for logs.

### Deployment

The project includes a deployment script for Raspberry Pi. To deploy, run the deployment script:

```bash
./deploy.sh
```

### Configuration Validation

The application validates all configuration values on startup. If any configuration is invalid, the application will exit with a descriptive error message. Common validation checks include:

- GPIO pin must be between 1-40
- Debounce delay must be greater than 0
- Monitor type must be either "gpio" or "web"
- Port numbers must be valid
- Log levels must be valid (trace, debug, info, warn, error)

[jetpham]: https://jetpham.com/
[gpio-pullup]: https://raspberrypi.stackexchange.com/questions/4569/what-is-a-pull-up-resistor-what-does-it-do-and-why-is-it-needed
[rppal-docs]: https://docs.rs/rppal/latest/rppal/gpio/struct.InputPin.html#method.set_async_interrupt
[rust-install]: https://www.rust-lang.org/tools/install
[rp02w]: https://www.raspberrypi.com/products/raspberry-pi-zero-2-w/
[cross-install]: https://github.com/cross-rs/cross
