#!/bin/bash
set -e

# TSLM Client Installation Script
# Usage: curl -sSL https://raw.githubusercontent.com/terminal-stream/last-mile/main/scripts/install.sh | bash

REPO="terminal-stream/last-mile"
BINARY_NAME="tslm-client"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Detect OS and Architecture
detect_platform() {
    local os
    local arch

    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os" in
        linux)
            OS="linux"
            ;;
        darwin)
            OS="darwin"
            ;;
        *)
            error "Unsupported operating system: $os"
            ;;
    esac

    case "$arch" in
        x86_64|amd64)
            ARCH="amd64"
            ;;
        aarch64|arm64)
            ARCH="arm64"
            ;;
        *)
            error "Unsupported architecture: $arch"
            ;;
    esac

    PLATFORM="${OS}-${ARCH}"
    info "Detected platform: $PLATFORM"
}

# Get latest release version
get_latest_version() {
    info "Fetching latest release version..."

    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

    if [ -z "$VERSION" ]; then
        error "Failed to fetch latest version"
    fi

    info "Latest version: $VERSION"
}

# Download binary
download_binary() {
    local asset_name="${BINARY_NAME}-${PLATFORM}"
    local download_url="https://github.com/${REPO}/releases/download/${VERSION}/${asset_name}"
    local checksum_url="${download_url}.sha256"

    info "Downloading ${asset_name}..."

    # Create temporary directory
    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT

    # Download binary
    if ! curl -fsSL "$download_url" -o "${TMP_DIR}/${BINARY_NAME}"; then
        error "Failed to download binary from $download_url"
    fi

    # Download and verify checksum
    info "Verifying checksum..."
    if curl -fsSL "$checksum_url" -o "${TMP_DIR}/${BINARY_NAME}.sha256" 2>/dev/null; then
        (
            cd "$TMP_DIR"
            if command -v sha256sum >/dev/null 2>&1; then
                sha256sum -c "${BINARY_NAME}.sha256" || error "Checksum verification failed"
            elif command -v shasum >/dev/null 2>&1; then
                shasum -a 256 -c "${BINARY_NAME}.sha256" || error "Checksum verification failed"
            else
                warn "sha256sum not found, skipping checksum verification"
            fi
        )
    else
        warn "Checksum file not available, skipping verification"
    fi

    # Make binary executable
    chmod +x "${TMP_DIR}/${BINARY_NAME}"

    DOWNLOADED_BINARY="${TMP_DIR}/${BINARY_NAME}"
}

# Install binary
install_binary() {
    info "Installing ${BINARY_NAME} to ${INSTALL_DIR}..."

    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"

    # Move binary to install directory
    mv "$DOWNLOADED_BINARY" "${INSTALL_DIR}/${BINARY_NAME}"

    info "Installation complete!"
}

# Check if install directory is in PATH
check_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "Install directory $INSTALL_DIR is not in your PATH"
        echo ""
        echo "Add the following line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo ""
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
        echo ""
    fi
}

# Show usage instructions
show_usage() {
    echo ""
    echo "Installation successful! 🎉"
    echo ""
    echo "Quick start:"
    echo "  ${BINARY_NAME} --help                           # Show help"
    echo "  ${BINARY_NAME} test                             # Run test scenario"
    echo "  ${BINARY_NAME} subscribe test-channel           # Subscribe to a channel"
    echo "  ${BINARY_NAME} create-channel my-channel        # Create a channel"
    echo "  ${BINARY_NAME} publish my-channel \"Hello\"      # Publish a message"
    echo ""
    echo "For more information, visit: https://github.com/${REPO}"
    echo ""
}

# Main installation flow
main() {
    info "TSLM Client Installer"
    info "====================="
    echo ""

    detect_platform
    get_latest_version
    download_binary
    install_binary
    check_path
    show_usage
}

main "$@"
