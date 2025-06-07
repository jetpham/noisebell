# Noisebell

A switch monitoring system that detects circuit state changes via GPIO and sends webhook notifications to configured endpoints.

This is build by Jet Pham to be used at Noisebridge to replace their old discord status bot

## Features

- GPIO circuit monitoring with configurable pin
>TODO: - Webhook notifications with retry mechanism 
>TODO: - REST API for managing webhook endpoints 
- Daily rotating log files
- Cross-compilation support for Raspberry Pi deployment
> Temporarialy calls the discord bot directly
- Debouncing using a finite state machine

## Requirements

- Rust toolchain
- Raspberry Pi (tested on aarch64)
- For development: Cross-compilation tools (for `cross` command)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/noisebell.git
cd noisebell
```

2. Build the project:
```bash
cargo build --release
```

## Deployment

The project includes a deployment script for Raspberry Pi. To deploy:

1. Ensure you have cross-compilation tools installed:
```bash
cargo install cross
```

2. Run the deployment script:
```bash
./deploy.sh
```

This will:
- Cross-compile the project for aarch64
- Copy the binary and configuration to your Raspberry Pi
- Set appropriate permissions

## Logging

Logs are stored in the `logs` directory with daily rotation for the past 7 days

## Configuration

The following parameters can be configured in `src/main.rs`:

### GPIO Settings
- `DEFAULT_GPIO_PIN`: The GPIO pin number to monitor (default: 17)
- `DEFAULT_POLL_INTERVAL_MS`: How frequently to check the GPIO pin state in milliseconds (default: 100ms)
- `DEFAULT_DEBOUNCE_DELAY_SECS`: How long the switch must remain in a stable state before triggering a change, in seconds (default: 5s)

### Discord Settings
The following environment variables must be set:
- `DISCORD_TOKEN`: Your Discord bot token
- `DISCORD_CHANNEL_ID`: The ID of the channel where status updates will be posted

### Logging Settings
- `LOG_DIR`: Directory where log files are stored (default: "logs")
- `LOG_PREFIX`: Prefix for log filenames (default: "noisebell")
- `LOG_SUFFIX`: Suffix for log filenames (default: "log")
- `MAX_LOG_FILES`: Maximum number of log files to keep (default: 7)

To modify these settings:
1. Edit the constants in `src/main.rs`
2. Rebuild the project
3. For Discord keys and channel id, ensure the environment variables are set before running the bot (Done for you in deploy.sh)