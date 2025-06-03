#!/bin/bash

# Exit on error
set -e

echo "Building for Raspberry Pi..."
cargo zigbuild --release --target aarch64-unknown-linux-gnu

echo "Copying to Raspberry Pi..."
scp target/aarch64-unknown-linux-gnu/release/noisebell noisebridge@noisebell.local:~/

echo "Setting permissions and restarting service..."
ssh noisebridge@noisebell.local "chmod +x ~/noisebell "

echo "Deployment complete!" 
