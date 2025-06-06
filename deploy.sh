#!/bin/bash

# Exit on error
set -e

echo "Building for Raspberry Pi..."
cross build --release --target aarch64-unknown-linux-gnu

echo "Copying to Raspberry Pi..."
scp target/aarch64-unknown-linux-gnu/release/noisebell noisebridge@noisebell.local:~/noisebell/
scp endpoints.json noisebridge@noisebell.local:/home/noisebridge/noisebell/endpoints.json

echo "Setting permissions"
ssh noisebridge@noisebell.local "chmod +x ~/noisebell/noisebell"

echo "Deployment complete!" 
