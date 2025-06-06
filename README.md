# Noisebell

A switch monitoring system that detects circuit state changes via GPIO and sends webhook notifications to configured endpoints.

This is build by Jet Pham to be used at Noisebridge to replace their old discord status bot

## Features

- GPIO circuit monitoring with configurable pin
- Webhook notifications with retry mechanism
- REST API for managing webhook endpoints
- Daily rotating log files
- Cross-compilation support for Raspberry Pi deployment

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

## Configuration

### GPIO Pin
The default GPIO pin is set to 17. You can modify this in `src/main.rs`.

### Webhook Endpoints
Webhook endpoints are stored in `endpoints.json`. The file should follow this format:
```json
{
  "endpoints": [
    {
      "url": "https://your-webhook-url.com",
      "description": "Description of this endpoint"
    }
  ]
}
```

## Usage

1. Start the server:
```bash
./target/release/noisebell
```

The server will:
- Start listening on `127.0.0.1:8080`
- Begin monitoring the configured GPIO pin
- Send webhook notifications when circuit state changes

### API Endpoints

#### Add Webhook Endpoint
```bash
curl -X POST http://localhost:8080/endpoints \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://your-webhook-url.com",
    "description": "My webhook"
  }'
```

### Webhook Payload Format
When a circuit state change is detected, the following JSON payload is sent to all configured endpoints:
```json
{
  "event_type": "circuit_state_change",
  "timestamp": "2024-03-21T12:34:56Z",
  "new_state": "open"  // or "closed"
}
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