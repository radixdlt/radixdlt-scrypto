#!/bin/bash

# Print commands and exit on errors
set -ex

# Versions to install
LLVM_VERSION=18
RUST_VERSION=1.81.0
RADIX_CLI_VERSION=1.3.0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Starting installation process...${NC}"

# Function to check if a command was successful
check_status() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ $1 successful${NC}"
    else
        echo -e "${RED}✗ $1 failed${NC}"
        exit 1
    fi
}

# Install Xcode Command Line Tools
echo -e "\n${BLUE}Installing Xcode Command Line Tools...${NC}"
xcode-select --install 2>/dev/null || true
check_status "Xcode Command Line Tools installation"

# Install cmake and LLVM
echo -e "\n${BLUE}Installing cmake and LLVM...${NC}"
brew install cmake llvm@$LLVM_VERSION
check_status "cmake and LLVM installation"

# Detect shell and configure appropriate rc file
SHELL_CONFIG=""
if [[ "$SHELL" == */bin/zsh ]]; then
    SHELL_CONFIG="$HOME/.zshrc"
elif [[ "$SHELL" == */bin/bash ]]; then
    SHELL_CONFIG="$HOME/.profile"
else
    echo -e "${RED}Unsupported shell: $SHELL${NC}"
    exit 1
fi

# Add LLVM to PATH
echo -e "\n${BLUE}Configuring LLVM in $SHELL_CONFIG...${NC}"
if ! grep -q "$(brew --prefix llvm@${LLVM_VERSION})/bin" "$SHELL_CONFIG"; then
    echo 'PATH="$(brew --prefix llvm@'$LLVM_VERSION')/bin:$PATH"' >> "$SHELL_CONFIG"
fi
check_status "LLVM path configuration"

# Install Rust
echo -e "\n${BLUE}Installing Rust...${NC}"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain=$RUST_VERSION -y
check_status "Rust installation"

# Source cargo environment
echo -e "\n${BLUE}Enabling cargo in current shell...${NC}"
source "$HOME/.cargo/env"
check_status "Cargo environment setup"

# Add WebAssembly target
echo -e "\n${BLUE}Adding WebAssembly target...${NC}"
rustup target add wasm32-unknown-unknown
check_status "WebAssembly target installation"

# Install Radix Engine Simulator and CLI tools
echo -e "\n${BLUE}Installing Radix Engine Simulator and CLI tools...${NC}"
cargo install --force radix-clis@$RADIX_CLI_VERSION
check_status "Radix tools installation"

echo -e "\n${GREEN}Installation complete! Please restart your terminal or run:${NC}"
echo -e "source $SHELL_CONFIG"
