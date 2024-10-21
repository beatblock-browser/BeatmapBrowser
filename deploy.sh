#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

echo "Starting deployment..."

rustup default stable

SSH_KEY="/var/lib/webhook/.ssh/id_rsa"

# Set GIT_SSH_COMMAND to use the specified SSH key
export GIT_SSH_COMMAND="ssh -i $SSH_KEY -o StrictHostKeyChecking=no"

# Pull the latest code
git pull origin master

# Build the application
cargo build --release

# Restart the systemd service
sudo systemctl restart rustapp.service

echo "Deployment completed successfully."