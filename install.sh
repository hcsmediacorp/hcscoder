#!/usr/bin/env bash
# hcscoder Install Script for macOS and Linux
# 
# This script installs hcscoder to /usr/local/bin (or $HOME/.local/bin)
# and sets up the configuration directory.
#
# Usage: curl -fsSL https://raw.githubusercontent.com/hcsmediacorp/hcscoder/main/install.sh | bash
#
# Build from source instead (100% Rust, no GitHub release binary required):
#   git clone https://github.com/hcsmediacorp/hcscoder.git && cd hcscoder
#   cargo install --path . --locked
#   hcscoder-setup && hcscoder chat
#
# MIT License (c) 2026 hcsmedia
# Attribution to hcsmedia is mandatory for all modifications and distributions.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="hcsmediacorp/hcscoder"
INSTALL_DIR="/usr/local/bin"
LOCAL_INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/hcscoder"
DATA_DIR="$HOME/.local/share/hcscoder"

# Detect architecture
detect_arch() {
    local arch
    arch=$(uname -m)
    case "$arch" in
        x86_64)
            echo "x86_64"
            ;;
        arm64|aarch64)
            echo "aarch64"
            ;;
        *)
            echo_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

# Detect OS
detect_os() {
    local os
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    case "$os" in
        linux)
            echo "unknown-linux-gnu"
            ;;
        darwin)
            echo "apple-darwin"
            ;;
        *)
            echo_error "Unsupported OS: $os"
            exit 1
            ;;
    esac
}

echo_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

echo_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

echo_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

echo_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Check if running as root (for system-wide install)
check_permissions() {
    if [[ $EUID -ne 0 ]] && [[ "$INSTALL_TYPE" != "local" ]]; then
        echo_warning "Not running as root. Will install to $LOCAL_INSTALL_DIR instead."
        INSTALL_DIR="$LOCAL_INSTALL_DIR"
        INSTALL_TYPE="local"
    fi
}

# Create necessary directories
create_directories() {
    echo_info "Creating directories..."
    
    if [[ "$INSTALL_TYPE" == "local" ]]; then
        mkdir -p "$LOCAL_INSTALL_DIR"
        if ! echo "$PATH" | grep -q "$LOCAL_INSTALL_DIR"; then
            echo_warning "Adding $LOCAL_INSTALL_DIR to PATH"
            echo "" >> "$HOME/.bashrc"
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
            echo "" >> "$HOME/.zshrc" 2>/dev/null || true
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc" 2>/dev/null || true
        fi
    else
        mkdir -p "$INSTALL_DIR"
    fi
    
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$DATA_DIR"
    
    echo_success "Directories created"
}

# Download and install binary
install_binary() {
    local arch
    local os
    local version="${VERSION:-latest}"
    local download_url
    
    arch=$(detect_arch)
    os=$(detect_os)
    
    echo_info "Detecting platform: $os ($arch)"
    
    # Get latest release if version not specified
    if [[ "$version" == "latest" ]]; then
        version=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    fi
    
    echo_info "Installing version: $version"
    
    # Construct download URL
    local binary_name="hcscoder-$version-$arch-$os"
    download_url="https://github.com/$REPO/releases/download/$version/$binary_name"
    
    echo_info "Downloading from: $download_url"
    
    # Download binary
    if ! curl -fsSL "$download_url" -o "/tmp/hcscoder"; then
        echo_error "Failed to download binary"
        echo_error "Please check if the release exists for your platform"
        exit 1
    fi
    
    # Make executable and move to install directory
    chmod +x "/tmp/hcscoder"
    
    if [[ "$INSTALL_TYPE" == "local" ]]; then
        mv "/tmp/hcscoder" "$LOCAL_INSTALL_DIR/hcscoder"
    else
        mv "/tmp/hcscoder" "$INSTALL_DIR/hcscoder"
    fi
    
    echo_success "Binary installed to $(which hcscoder)"
}

# Create default config file
create_default_config() {
    local config_file="$CONFIG_DIR/config.toml"
    
    if [[ ! -f "$config_file" ]]; then
        echo_info "Creating default configuration..."
        cat > "$config_file" << 'EOF'
# hcscoder Configuration
# MIT License (c) 2026 hcsmedia

# OpenRouter API Key (get yours at https://openrouter.ai/keys)
# You can also set this via environment variable: OPENROUTER_API_KEY
# api_key = ""

# Default model to use
model = "anthropic/claude-sonnet-4-20250514"

# Temperature for completions (0.0 - 2.0)
temperature = 0.7

# Maximum tokens for completions
max_tokens = 4096

# Enable verbose logging
verbose = false

# Working directory (default: current directory)
# working_dir = ""
EOF
        echo_success "Default configuration created at $config_file"
    else
        echo_info "Configuration already exists at $config_file"
    fi
}

# Verify installation
verify_installation() {
    echo_info "Verifying installation..."
    
    if command -v hcscoder &> /dev/null; then
        local version
        version=$(hcscoder --version 2>&1 || echo "unknown")
        echo_success "hcscoder installed successfully! Version: $version"
        echo ""
        echo "To get started:"
        echo "  1. Set your OpenRouter API key:"
        echo "     export OPENROUTER_API_KEY='your-key-here'"
        echo "  2. Or edit the config file: $CONFIG_DIR/config.toml"
        echo "  3. Run: hcscoder --help"
        echo ""
    else
        echo_error "Installation verification failed"
        echo_error "Please ensure $(dirname "$(realpath "$0")") is in your PATH"
        exit 1
    fi
}

# Main installation function
main() {
    echo ""
    echo "=========================================="
    echo "  hcscoder Installer"
    echo "  MIT License (c) 2026 hcsmedia"
    echo "=========================================="
    echo ""
    
    INSTALL_TYPE="${INSTALL_TYPE:-system}"
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --local)
                INSTALL_TYPE="local"
                shift
                ;;
            --version)
                VERSION="$2"
                shift 2
                ;;
            --help)
                echo "Usage: $0 [--local] [--version X.Y.Z]"
                echo ""
                echo "Options:"
                echo "  --local     Install to ~/.local/bin instead of /usr/local/bin"
                echo "  --version   Install specific version (default: latest)"
                echo "  --help      Show this help message"
                exit 0
                ;;
            *)
                echo_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    check_permissions
    create_directories
    install_binary
    create_default_config
    verify_installation
    
    echo ""
    echo_success "Installation complete!"
    echo ""
    echo "Legal Notice:"
    echo "  MIT License (c) 2026 hcsmedia"
    echo "  Attribution to hcsmedia is mandatory for all modifications and distributions."
    echo ""
}

# Run main function
main "$@"
