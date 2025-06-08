#!/bin/bash

# Exit on error
set -e

echo "Building for Raspberry Pi..."
cross build --release --target aarch64-unknown-linux-gnu

# Check if Discord credentials are already set
if [ -z "$DISCORD_TOKEN" ]; then
    echo "Please enter your Discord bot token:"
    read -s DISCORD_TOKEN
fi

if [ -z "$DISCORD_CHANNEL_ID" ]; then
    echo "Please enter your Discord channel ID:"
    read -s DISCORD_CHANNEL_ID
fi

# Create service file with credentials
cat > noisebell.service << EOL
[Unit]
Description=Noisebell Discord Notification Service
After=network.target

[Service]
Type=simple
User=noisebridge
WorkingDirectory=/home/noisebridge
Environment=DISCORD_TOKEN=${DISCORD_TOKEN}
Environment=DISCORD_CHANNEL_ID=${DISCORD_CHANNEL_ID}
ExecStart=/home/noisebridge/noisebell
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
EOL

echo "Copying to Raspberry Pi..."
# Copy files
 scp target/aarch64-unknown-linux-gnu/release/noisebell noisebridge@noisebell.local:~/
scp noisebell.service noisebridge@noisebell.local:~/

echo "Setting up service..."
# Deploy service
ssh noisebridge@noisebell.local "sudo cp ~/noisebell.service /etc/systemd/system/ && \
    sudo systemctl daemon-reload && \
    sudo systemctl enable noisebell && \
    sudo systemctl restart noisebell"

# Clean up local service file
rm noisebell.service

echo "Deployment complete!"
echo "You can check the service status with: ssh noisebridge@noisebell.local 'sudo systemctl status noisebell'" 
