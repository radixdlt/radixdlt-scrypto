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

# Check if Homebrew is installed, install if it's not
echo -e "\n${BLUE}Checking for Homebrew...${NC}"
if ! command -v brew &>/dev/null; then
    echo -e "${BLUE}Homebrew not found. Installing Homebrew...${NC}"
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    check_status "Homebrew installation"

    # Add Homebrew to PATH
    echo -e "\n${BLUE}Configuring Homebrew in $SHELL_CONFIG...${NC}"
    if [[ "$(uname -m)" == "arm64" ]]; then
        echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> "$SHELL_CONFIG"
        eval "$(/opt/homebrew/bin/brew shellenv)"
    else
        echo 'eval "$(/usr/local/bin/brew shellenv)"' >> "$SHELL_CONFIG"
        eval "$(/usr/local/bin/brew shellenv)"
    fi
    check_status "Homebrew path configuration"
else
    echo -e "${GREEN}Homebrew is already installed.${NC}"
fi

# Install cmake and LLVM
echo -e "\n${BLUE}Installing cmake and LLVM...${NC}"
brew install cmake --formula llvm@$LLVM_VERSION
check_status "cmake and LLVM installation"

# Add LLVM to PATH
echo -e "\n${BLUE}Configuring LLVM in $SHELL_CONFIG...${NC}"
LLVM_PATH_LINE='export PATH="$(brew --prefix llvm@'"$LLVM_VERSION"')/bin:$PATH"'
if ! grep -Fxq "$LLVM_PATH_LINE" "$SHELL_CONFIG"; then
    echo "$LLVM_PATH_LINE" >> "$SHELL_CONFIG"
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

# Verify installations
echo -e "\n${BLUE}Verifying installations...${NC}"
echo -e "Versions installed:"
echo -e "LLVM: $(llvm-config --version)"
echo -e "Clang: $(clang --version | head -n 1)"
echo -e "Rust: $(rustc --version)"
echo -e "Cargo: $(cargo --version)"
echo -e "Radix CLI: $(scrypto --version)"

echo -e "\n${GREEN}Installation complete! Please restart your terminal or run:${NC}"
echo -e "source $SHELL_CONFIG"
