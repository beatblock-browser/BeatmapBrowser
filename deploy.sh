#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

echo "Starting deployment..."

# Pull the latest code
git pull origin master

# Build the application
cargo build --release

# Restart the systemd service
sudo systemctl restart rustapp.service

echo "Deployment completed successfully."