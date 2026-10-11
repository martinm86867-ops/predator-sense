#!/usr/bin/env bash
# Predator Sense - Automated Source Build & System Installer
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
GUI_DIR="${REPO_ROOT}/predator-sense-gui"

echo "=================================================="
echo "  Predator Sense for Linux - Installer Setup"
echo "=================================================="

# Detect elevation helper
ELEVATE=""
if [ "$(id -u)" -ne 0 ]; then
    if command -v sudo >/dev/null 2>&1; then
        ELEVATE="sudo"
    elif command -v pkexec >/dev/null 2>&1; then
        ELEVATE="pkexec"
    else
        echo "Error: Root privileges required. Please install sudo or run as root." >&2
        exit 1
    fi
fi

# Detect package manager & install build prerequisites if needed
install_dependencies() {
    echo "Checking system build dependencies..."
    if command -v apt-get >/dev/null 2>&1; then
        echo "Installing Debian/Ubuntu dependencies via apt-get..."
        $ELEVATE apt-get update -qq || true
        $ELEVATE apt-get install -y \
            libgtk-4-dev libadwaita-1-dev pkg-config build-essential \
            gcc make dkms curl tar sudo acl "linux-headers-$(uname -r)"
    elif command -v dnf >/dev/null 2>&1; then
        echo "Installing Fedora/RHEL dependencies via dnf..."
        $ELEVATE dnf install -y \
            gtk4-devel libadwaita-devel pkgconf-pkg-config gcc make dkms curl tar sudo acl "kernel-devel-$(uname -r)"
    elif command -v pacman >/dev/null 2>&1; then
        echo "Installing Arch Linux dependencies via pacman..."
        $ELEVATE pacman -S --noconfirm --needed \
            gtk4 libadwaita pkgconf gcc make dkms curl tar sudo acl linux-headers
    elif command -v zypper >/dev/null 2>&1; then
        echo "Installing openSUSE dependencies via zypper..."
        $ELEVATE zypper --non-interactive install \
            gtk4-devel libadwaita-devel pkgconf-pkg-config gcc make dkms curl tar sudo acl "kernel-default-devel"
    fi
}

# Ensure Cargo/Rust is installed
ensure_rust() {
    if ! command -v cargo >/dev/null 2>&1; then
        echo "Cargo not found. Installing Rust toolchain via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
    fi
}

# Check if prebuilt installer is already built or needs building
build_and_install() {
    install_dependencies
    ensure_rust

    echo "Building Predator Sense Installer multicall binary..."
    cargo build --release --manifest-path "${GUI_DIR}/installer/Cargo.toml"

    echo "Building Predator Sense GUI desktop application..."
    cargo build --release --manifest-path "${GUI_DIR}/Cargo.toml"

    echo "Executing Predator Sense system installation..."
    $ELEVATE "${GUI_DIR}/installer/target/release/predator-sense-installer" --install
}

build_and_install
