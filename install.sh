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

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# --- Main Script ---
main() {
    info "Starting the quick setup for adaptiveRouting..."

    # 1. Check for dependencies
    info "Checking for required dependencies..."
    if ! command_exists git; then
        error "Git is not installed. Please install it first."
        error "Example for Debian/Ubuntu: sudo apt update && sudo apt install -y git"
        exit 1
    fi

    if ! command_exists curl; then
        error "curl is not installed. Please install it first."
        error "Example for Debian/Ubuntu: sudo apt update && sudo apt install -y curl"
        exit 1
    fi

    # Check and install build-essential on Debian-based systems
    if [ -f /etc/debian_version ]; then
        if ! dpkg -s build-essential >/dev/null 2>&1; then
            info "'build-essential' package not found. Attempting to install..."
            sudo apt-get update
            sudo apt-get install -y build-essential
        fi
    fi

    # 2. Install Rust if not already installed
    if ! command_exists cargo; then
        info "Rust is not installed. Installing Rust via rustup..."
        # The -y flag automates the installation
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # Add cargo to the current shell's PATH
        source "$HOME/.cargo/env"
        info "Rust has been installed successfully."
    else
        info "Rust is already installed."
    fi

    # 3. Clone repository
    REPO_DIR="adaptiveRouting"
    if [ -d "$REPO_DIR" ]; then
        info "Directory '$REPO_DIR' already exists. Skipping clone."
    else
        info "Cloning the adaptiveRouting repository..."
        git clone https://github.com/NextRouter/adaptiveRouting.git
    fi
    
    cd "$REPO_DIR"

    # 4. Build the project
    info "Building the project... (This may take a while)"
    cargo build --release

    info "\n🎉 Setup complete! 🎉"
    echo
    info "To run the application manually, use the following command:"
    echo "  sudo ./target/release/adaptiveRouting"
    echo
    
    if [ -f "./setup-systemd.sh" ]; then
        info "Running the systemd setup script..."
        sudo ./setup-systemd.sh
    else
        error "'setup-systemd.sh' not found. Please run it manually after setup."
    fi

}

# Run the main function
main
