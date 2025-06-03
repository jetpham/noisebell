# Noisebell

Noisebridge's better doorbell

## Explination

Noisebridge's old doorbell bot last sent a message on 5/4/25 and has been dead since. There's some thing that could be done to not just revive, it but make it more reliable, accessable, and exellent than ever before.

## Old Doorbell

The old boisebridge switch is a large breaker switch connected to a ESP-WROOM-32 ESP32 ESP-32S Development Board which has wifi support.

There is no avalible documentation on how this device works and the code is not open source.

## New Doorbell Features

Noisebell keeps the breaker switch used already and continues it with more documentation, uptime, transparency, and coolness.

- Status information is distributed to multiple channels rather than just discord:

> - Discord Bot
> - RSS
> - Noisebell status website
> - Telegram Channel

- The status website also shows information about the device and previous hours data

- Automatic security updates with `unattended-upgrades`

- Remote development via `ssh`

- Automatic running with `Systemd`

- All ran on a Raspberry Pi Zero 2 W in a cute case

- Secret managment by uhhhhh. Be Exellent?

## This Repo

This is a monorepo of all of the different services that this project deploys and uses.

## Contributing

Please Do! Any and all pull requests are welcome! I am `@jetpham` on the discord and I'd love to help add any features you think we should add.

## Inspiration

https://github.com/FireflyHacker/discord_room_alert_bot
https://github.com/0xjmux/room_alert_bot
