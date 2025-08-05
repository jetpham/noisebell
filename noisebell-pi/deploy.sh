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
# Debug remote directory status
ssh noisebridge@noisebell.local "pwd && ls -la ~/ && echo 'Directory permissions:' && stat -c '%A %a %n' ~/"
# Remove existing files
ssh noisebridge@noisebell.local "rm -f /home/noisebridge/noisebell /home/noisebridge/noisebell.service"
# Copy files with absolute paths
scp -v target/aarch64-unknown-linux-gnu/release/noisebell noisebridge@noisebell.local:/home/noisebridge/noisebell
scp -v noisebell.service noisebridge@noisebell.local:/home/noisebridge/noisebell.service

echo "Setting up service..."
# Deploy service
ssh noisebridge@noisebell.local "sudo cp /home/noisebridge/noisebell.service /etc/systemd/system/ && \
    sudo systemctl daemon-reload && \
    sudo systemctl enable noisebell && \
    sudo systemctl restart noisebell"

# Clean up local service file
rm noisebell.service

echo "Deployment complete!"
echo "You can check the service status with: ssh noisebridge@noisebell.local 'sudo systemctl status noisebell'" 
