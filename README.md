# Noisebridge Open Status Webhook

Noisebridge's better doorbell

## Explination

Noisebridge's old doorbell bot last sent a message on 5/4/25 and has been dead since. There's some thing that could be done to not just revive, it but make it more reliable, accessable, and exellent than ever before.

## Old Doorbell

The old boisebridge switch is a large breaker switch connected to a ESP-WROOM-32 ESP32 ESP-32S Development Board which has wifi support.

There is no avalible documentation on how this device works and the code is not open source.

## New Doorbell Features

Noisebell's infrastructure is based on webhooks. When the switch flips, it sends out 
Noisebell keeps the breaker switch used already and continues it with more documentation, uptime, transparency, and coolness.

- Automatic security updates with `unattended-upgrades`

- Remote development via `ssh`

- Automatic running with `Systemd`

- REST API for actively requesting status and updating webhooks

- Webhooks update the current status and startup and shutdown

- All ran on a Raspberry Pi Zero 2 W in a cute case

## Channels

The webhooks can connect to any internet connected program. We have planned to connect Noisebridge Open Status Webhook to:

- A Discord Bot
- A Telegram Channel
- An Email Chain
- A RSS feed
- The 2nd floor open neon sign
- Home automation light switches
- 

## This Repo

This Repo is the infrastructure for reacting to switch updates and sending out webhooks and responcding to API requests for current status.

## API

### Webhooks 

Will contain a webhook to update the status of the switch. Containing:
- The new state
- The time the switch is switched

Another webhook to indicate startup and planning shutdowns

### REST Endpoints

> Will require an API KEY

GET noisebridge open status
GET Ping pong to check if active
PUT add webhook
DELETE webhook

## Contributing

Please Do! Any and all pull requests are welcome! I am `@jetpham` on the discord and I'd love to help add any features you think we should add.

## Inspiration

https://github.com/FireflyHacker/discord_room_alert_bot
https://github.com/0xjmux/room_alert_bot

## Access
ssh access is with with keys and is currently only with Jet
with physical access, the username is `noisebridge` and the password is `flaschentaschen`

## ssh
be connected onto the `Noisebridge Cap` wifi
`ssh noisebridge@raspberrypi.local`