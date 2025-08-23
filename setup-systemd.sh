#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# --- Helper Functions ---
info() {
    echo -e "\033[34m[INFO]\033[0m $1"
}

error() {
    echo -e "\033[31m[ERROR]\033[0m $1" >&2
}

# --- Main Script ---
main() {
    info "Setting up systemd service for adaptiveRouting..."

    # Check if running with sudo
    if [ "$(id -u)" -ne 0 ]; then
        error "This script must be run with sudo. Please run 'sudo ./setup-systemd.sh'"
        exit 1
    fi

    # Check if the binary exists
    if [ ! -f "./target/release/adaptiveRouting" ]; then
        error "The application binary is not found. Please build the project first with 'cargo build --release'."
        exit 1
    fi

    # 1. Create the service file
    WORKDIR=$(pwd)
    SERVICE_FILE="/etc/systemd/system/adaptiverouting.service"

    info "Creating systemd service file at $SERVICE_FILE..."

    tee "$SERVICE_FILE" > /dev/null <<EOF
[Unit]
Description=Adaptive Routing Service
After=network.target
Wants=network.target

[Service]
Type=simple
User=root
Group=root
ExecStart=$WORKDIR/target/release/adaptiveRouting
WorkingDirectory=$WORKDIR
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security settings
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=false
ReadWritePaths=/tmp

[Install]
WantedBy=multi-user.target
EOF

    info "Service file created successfully."

    # 2. Reload, enable, and start the service
    info "Reloading systemd daemon..."
    systemctl daemon-reload

    info "Enabling adaptiveRouting service to start on boot..."
    systemctl enable adaptiverouting.service

    info "Starting adaptiveRouting service..."
    systemctl start adaptiverouting.service

    info "\n🎉 Systemd service setup complete! 🎉"
    echo
    info "The adaptiveRouting service is now running and will start automatically on boot."
    info "You can check the status of the service with:"
    echo "  sudo systemctl status adaptiverouting.service"
    info "You can view the logs with:"
    echo "  sudo journalctl -u adaptiverouting.service -f"
}

# Run the main function
main
