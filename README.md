# <img src="media/noisebell%20logo.png" width="100" alt="Noisebell Logo" style="vertical-align: middle; margin-right: 20px;"> Noisebell

A switch monitoring system that detects circuit state changes via GPIO and sends webhook notifications to configured endpoints.

This is build by [Jet Pham][jetpham] to be used at Noisebridge to replace their old discord status bot

## Features

- GPIO circuit monitoring with configurable pin
- Webhook notifications with retry mechanism
- REST API for managing webhook endpoints
- API endpoints for actively polling status and health
- Daily rotating log files
- Cross-compilation support for Raspberry Pi deploymentk
- Software Deboucing to prevent noisy switch detection

## How it works

This project is the core of a system of services that all-together send and manage notifications about the noisebridge open status.

In this service, we manage two systems that are a source of truth for the status of the noisebridge open status aswell as this services' status.

### GPIO and Physical Tech

We interact directly over a [GPIO pin in a pull-up configuration][gpio-pullup] to read whether a circuit has been closed with a switch. This is an extremely simple circuit that will internally call a callback function and send out the webhooks when the state of the circuit changes

<details>
<summary>Debouncing</summary>

When a switch changes state, it can bounce and create multiple rapid signals. Debouncing adds a delay to wait for the signal to settle, ensuring we only detect one clean state change instead of multiple false ones.

We do debouncing with software via [`set_async_interupt`][rppal-docs] which handles software debounce for us.

</details>

### Logging

Logs are stored in the `logs` directory with daily rotation for the past 7 days

### API

The service exposes a REST API for monitoring and managing webhooks. All endpoints return JSON responses with a `status` field indicating success or error.

#### Webhook Management

> [!CAUTION]
> The webhook management endpoints shown below are examples and not yet implemented.

- `GET /webhooks` - List all configured webhooks

  ```json
  {
    "status": "success",
    "data": {
      "webhooks": [
        {
          "url": "https://example.com/webhook",
          "enabled": true
        }
      ]
    }
  }
  ```

- `POST /webhooks` - Add a new webhook

  ```json
  {
    "status": "success",
    "message": "Webhook added successfully",
    "data": {
      // Your webhook configuration
    }
  }
  ```

- `PUT /webhooks` - Update an existing webhook

  ```json
  {
    "status": "success",
    "message": "Webhook updated successfully",
    "data": {
      // Updated webhook configuration
    }
  }
  ```

- `DELETE /webhooks` - Remove a webhook

  ```json
  {
    "status": "success",
    "message": "Webhook deleted successfully",
    "data": {
      // Deleted webhook configuration
    }
  }
  ```

#### Status Endpoints

- `GET /status` - Get the current state of the monitored circuit

  ```json
  {
    "status": "success",
    "data": {
      "state": "open" // or "closed"
    }
  }
  ```

- `GET /health` - Get detailed health metrics about the service

This data is parsed from the `systemctl show $SERVICE_NAME` command.

To see what data is possible, see `org.freedesktop.systemd1(5)`

  ```json
  {
    "status": "success",
    "data": {
      "ActiveState": "active",
      "SubState": "running",
      "MainPID": 1234,
      "TasksCurrent": 1,
      "CPUUsageSeconds": 120,
      "MemoryCurrent": 1024000,
      "Uptime": "2d 5h 30m"
    }
  }
  ```

The health endpoint provides detailed system metrics including:

- Service state and status
- Process ID and task count
- CPU and memory usage
- Service uptime

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
- `cross` (Install [Cross][cross-install])

### Deployment

The project includes a deployment script for Raspberry Pi. To deploy, run the deployment script:

```bash
./deploy.sh
```

This will:

- Cross-compile the project for `aarch64`
- Copy the binary and configuration to your Raspberry Pi
- Chmod the binary
- Restart the [`systemd`][systemd] service

### Configuration

The following parameters can be configured in `src/main.rs`:

To modify these settings:

1. Edit the constants in `src/main.rs`
2. Rebuild the project

#### GPIO Settings

- `DEFAULT_GPIO_PIN`: The GPIO pin number to monitor (default: 17)
- `DEFAULT_DEBOUNCE_DELAY_SECS`: How long the switch must remain in a stable state before triggering a change, in seconds (default: 5s)

#### API Settings

- `DEFAULT_API_PORT`: The port number for the API server (default: 3000)

#### Logging Settings

- `LOG_DIR`: Directory where log files are stored (default: "logs")
- `LOG_PREFIX`: Prefix for log filenames (default: "noisebell")
- `LOG_SUFFIX`: Suffix for log filenames (default: "log")
- `MAX_LOG_FILES`: Maximum number of log files to keep (default: 7)

[jetpham]: https://jetpham.com/
[gpio-pullup]: https://raspberrypi.stackexchange.com/questions/4569/what-is-a-pull-up-resistor-what-does-it-do-and-why-is-it-needed
[rppal-docs]: https://docs.rs/rppal/latest/rppal/gpio/struct.InputPin.html#method.set_async_interrupt
[rust-install]: https://www.rust-lang.org/tools/install
[rp02w]: https://www.raspberrypi.com/products/raspberry-pi-zero-2-w/
[cross-install]: https://github.com/cross-rs/cross
[systemd]: https://en.wikipedia.org/wiki/Systemd
