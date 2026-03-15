#!/bin/bash
#
# Axel installer script
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/lgalabru/axel/main/cli/scripts/install.sh | bash
#
# Options:
#   AXEL_VERSION=cli-v0.1.0  Install a specific version
#   AXEL_INSTALL_DIR=~/.local/bin  Install to a specific directory
#

set -euo pipefail

REPO="lgalabru/axel"
BINARY_NAME="axel"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}▸${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1" >&2; exit 1; }

# Detect platform
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)  os="linux" ;;
        Darwin*) os="darwin" ;;
        *)       error "Unsupported OS: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)  arch="x64" ;;
        arm64|aarch64) arch="arm64" ;;
        *)             error "Unsupported architecture: $(uname -m)" ;;
    esac

    echo "${os}-${arch}"
}

# Get latest CLI release version
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases" \
        | grep '"tag_name":' \
        | sed -E 's/.*"([^"]+)".*/\1/' \
        | grep '^cli-v' \
        | head -1
}

main() {
    local platform version install_dir tmp_dir

    platform=$(detect_platform)
    version="${AXEL_VERSION:-$(get_latest_version)}"
    install_dir="${AXEL_INSTALL_DIR:-${HOME}/.local/bin}"
    tmp_dir=$(mktemp -d)

    info "Installing ${BINARY_NAME} ${version} for ${platform}..."

    # Create install directory
    mkdir -p "${install_dir}"

    # Download and extract
    local url="https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}-${platform}.tar.gz"
    info "Downloading from ${url}..."

    curl -fsSL "${url}" | tar -xz -C "${tmp_dir}"

    # Install binary
    mv "${tmp_dir}/${BINARY_NAME}" "${install_dir}/${BINARY_NAME}"
    chmod +x "${install_dir}/${BINARY_NAME}"

    # Cleanup
    rm -rf "${tmp_dir}"

    success "Installed ${BINARY_NAME} to ${install_dir}/${BINARY_NAME}"

    # Check if install_dir is in PATH
    if [[ ":${PATH}:" != *":${install_dir}:"* ]]; then
        echo ""
        echo "Add the following to your shell profile (.bashrc, .zshrc, etc.):"
        echo ""
        echo "  export PATH=\"\${PATH}:${install_dir}\""
        echo ""
    fi

    # Verify installation
    if command -v "${BINARY_NAME}" &> /dev/null; then
        success "Run '${BINARY_NAME} --help' to get started"
    fi
}

main "$@"
