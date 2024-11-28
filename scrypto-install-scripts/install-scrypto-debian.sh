#!/bin/bash

# Exit on error
set -e

# Versions to install
LLVM_VERSION=18
RUST_VERSION=1.81.0
RADIX_CLI_VERSION=1.3.0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to log messages with colors
log() {
    local level=$1
    shift
    case $level in
        "INFO") echo -e "${BLUE}[INFO]${NC} $*" ;;
        "SUCCESS") echo -e "${GREEN}[SUCCESS]${NC} $*" ;;
        "ERROR") echo -e "${RED}[ERROR]${NC} $*" ;;
        "WARN") echo -e "${YELLOW}[WARN]${NC} $*" ;;
    esac
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check system dependencies
check_dependencies() {
    log "INFO" "Checking system dependencies..."
    
    # Check if we're on a Debian-based system
    if ! command_exists apt-get; then
        log "ERROR" "This script requires apt-get (Debian/Ubuntu). For other distributions, please modify the script accordingly."
        exit 1
    fi

    # Check if sudo is available
    if ! command_exists sudo; then
        log "ERROR" "This script requires sudo privileges."
        exit 1
    fi
}

# Function to install build essentials
install_build_essentials() {
    log "INFO" "Installing build essentials..."
    
    # Update package list
    sudo apt-get update
    
    # Install build essentials
    sudo apt-get install -y build-essential
    
    if [ $? -eq 0 ]; then
        log "SUCCESS" "Build essentials installed successfully"
    else
        log "ERROR" "Failed to install build essentials"
        exit 1
    fi
}

# Function to install LLVM
install_llvm() {
    log "INFO" "Installing LLVM..."

    # Download and install LLVM
    wget https://apt.llvm.org/llvm.sh
    chmod +x llvm.sh
    sudo ./llvm.sh $LLVM_VERSION
    rm llvm.sh

    if [ $? -eq 0 ]; then
        log "SUCCESS" "LLVM installed successfully"
    else
        log "ERROR" "Failed to install LLVM"
        exit 1
    fi
}

# Function to install Rust
install_rust() {
    log "INFO" "Installing Rust..."
    
    # Download and install Rust with specific toolchain
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain=$RUST_VERSION -y
    
    if [ $? -eq 0 ]; then
        log "SUCCESS" "Rust installed successfully"
    else
        log "ERROR" "Failed to install Rust"
        exit 1
    fi
}

# Function to setup Cargo environment
setup_cargo() {
    log "INFO" "Setting up Cargo environment..."
    
    # Source cargo environment
    source "$HOME/.cargo/env"
    
    if command_exists cargo; then
        log "SUCCESS" "Cargo environment setup successfully"
    else
        log "ERROR" "Failed to setup Cargo environment"
        exit 1
    fi
}

# Function to add WebAssembly target
add_wasm_target() {
    log "INFO" "Adding WebAssembly target..."
    
    rustup target add wasm32-unknown-unknown
    
    if [ $? -eq 0 ]; then
        log "SUCCESS" "WebAssembly target added successfully"
    else
        log "ERROR" "Failed to add WebAssembly target"
        exit 1
    fi
}

# Function to install Radix tools
install_radix_tools() {
    log "INFO" "Installing Radix Engine Simulator and CLI tools..."
    
    cargo install --force radix-clis@$RADIX_CLI_VERSION
    
    if [ $? -eq 0 ]; then
        log "SUCCESS" "Radix tools installed successfully"
    else
        log "ERROR" "Failed to install Radix tools"
        exit 1
    fi
}

# Function to add Clang to shell configuration
add_clang_to_shell_config() {
    # Determine the user's shell
    SHELL_NAME=$(basename "$SHELL")

    # Determine the shell configuration file based on the default shell
    case "$SHELL_NAME" in
        bash) SHELL_CONFIG="$HOME/.bashrc" ;;
        zsh) SHELL_CONFIG="$HOME/.zshrc" ;;
        ksh) SHELL_CONFIG="$HOME/.kshrc" ;;
        fish) SHELL_CONFIG="$HOME/.config/fish/config.fish" ;;
        *)
        log "ERROR" "Unsupported shell. Please add 'export CC=clang-${LLVM_VERSION}' to your shell configuration manually."
        exit 1
            ;;
    esac

    # The line to add
    EXPORT_LINE="export CC=clang-${LLVM_VERSION}"

    # Check if the line already exists to prevent duplicates
    if grep -Fxq "$EXPORT_LINE" "$SHELL_CONFIG"
    then
        log "INFO" "The CC variable is already set in $SHELL_CONFIG"
    else
        # Backup the shell configuration file
        cp "$SHELL_CONFIG" "${SHELL_CONFIG}.backup"

        # Append the export line to the shell configuration file
        echo "$EXPORT_LINE" >> "$SHELL_CONFIG"
        log "INFO" "Added '$EXPORT_LINE' to $SHELL_CONFIG"
    fi
}

# Main installation process
main() {
    log "INFO" "Starting installation process..."
    
    # Check system dependencies
    check_dependencies
    
    # Install components
    install_build_essentials
    install_llvm
    install_rust
    setup_cargo
    add_wasm_target
    install_radix_tools
    add_clang_to_shell_config
    
    # Final success message
    log "SUCCESS" "Installation completed successfully!"
    source $HOME/.cargo/env
    
    # Verify installations
    log "INFO" "Verifying installations..."
    echo -e "\nVersions installed:"
    echo -e "LLVM: $(llvm-config-${LLVM_VERSION} --version)"
    echo -e "Clang: $(clang-${LLVM_VERSION} --version | head -n 1)"
    echo -e "Rust: $(rustc --version)"
    echo -e "Cargo: $(cargo --version)"
    echo -e "Radix CLI: $(scrypto --version)\n"
 
    log "INFO" "Please restart your terminal or run:"
    echo "source $SHELL_CONFIG"
}

# Run main function
main
