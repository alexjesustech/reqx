#!/usr/bin/env bash
# reqx installer script
# Usage: curl -fsSL https://reqx.dev/install.sh | sh

set -euo pipefail

REQX_VERSION="${REQX_VERSION:-latest}"
REQX_INSTALL_DIR="${REQX_INSTALL_DIR:-$HOME/.local/bin}"
GITHUB_REPO="reqx/reqx"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

info() { echo -e "${CYAN}$1${NC}"; }
success() { echo -e "${GREEN}$1${NC}"; }
error() { echo -e "${RED}$1${NC}" >&2; exit 1; }

# Detect OS and architecture
detect_platform() {
    local os arch
    
    case "$(uname -s)" in
        Linux*)  os="unknown-linux-musl" ;;
        Darwin*) os="apple-darwin" ;;
        MINGW*|MSYS*|CYGWIN*) os="pc-windows-msvc" ;;
        *) error "Unsupported OS: $(uname -s)" ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) error "Unsupported architecture: $(uname -m)" ;;
    esac
    
    echo "${arch}-${os}"
}

# Get download URL
get_download_url() {
    local platform="$1"
    local version="$2"
    
    if [ "$version" = "latest" ]; then
        echo "https://github.com/${GITHUB_REPO}/releases/latest/download/reqx-${platform}.tar.gz"
    else
        echo "https://github.com/${GITHUB_REPO}/releases/download/v${version}/reqx-${platform}.tar.gz"
    fi
}

main() {
    info "Installing reqx ${REQX_VERSION}..."
    
    local platform
    platform=$(detect_platform)
    info "Detected platform: ${platform}"
    
    local download_url
    download_url=$(get_download_url "$platform" "$REQX_VERSION")
    info "Downloading from: ${download_url}"
    
    # Create install directory
    mkdir -p "$REQX_INSTALL_DIR"
    
    # Download and extract
    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap 'rm -rf "$tmp_dir"' EXIT
    
    if command -v curl &> /dev/null; then
        curl -fsSL "$download_url" | tar -xzf - -C "$tmp_dir"
    elif command -v wget &> /dev/null; then
        wget -qO- "$download_url" | tar -xzf - -C "$tmp_dir"
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
    
    # Install binary
    mv "$tmp_dir/reqx" "$REQX_INSTALL_DIR/reqx"
    chmod +x "$REQX_INSTALL_DIR/reqx"
    
    success "✓ reqx installed to ${REQX_INSTALL_DIR}/reqx"
    
    # Check if in PATH
    if ! echo "$PATH" | grep -q "$REQX_INSTALL_DIR"; then
        echo ""
        info "Add reqx to your PATH by adding this to your shell profile:"
        echo ""
        echo "  export PATH=\"\$PATH:$REQX_INSTALL_DIR\""
        echo ""
    fi
    
    # Verify installation
    if command -v reqx &> /dev/null || [ -x "$REQX_INSTALL_DIR/reqx" ]; then
        echo ""
        "$REQX_INSTALL_DIR/reqx" --version
        success "Installation complete!"
    fi
}

main "$@"
