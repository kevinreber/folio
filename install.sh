#!/usr/bin/env bash
#
# Folio Installation Script
#
# Usage:
#   curl -sSf https://raw.githubusercontent.com/kevinreber/folio/main/install.sh | sh
#
# Or with options:
#   curl -sSf https://raw.githubusercontent.com/kevinreber/folio/main/install.sh | sh -s -- --help
#
# This script will:
#   1. Check for Rust (install via rustup if missing)
#   2. Clone/download the repository
#   3. Build the release binary
#   4. Install to ~/.local/bin/
#   5. Optionally run initial configuration

set -e

# Configuration
FOLIO_REPO="https://github.com/kevinreber/folio.git"
FOLIO_INSTALL_DIR="${FOLIO_INSTALL_DIR:-$HOME/.local/bin}"
FOLIO_BUILD_DIR="${FOLIO_BUILD_DIR:-$HOME/.folio-install}"
FOLIO_VERSION="${FOLIO_VERSION:-main}"

# Colors for output (disable if not a terminal)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    BOLD=''
    NC=''
fi

# Logging functions
info() {
    echo -e "${BLUE}==>${NC} ${BOLD}$1${NC}"
}

success() {
    echo -e "${GREEN}==>${NC} ${BOLD}$1${NC}"
}

warn() {
    echo -e "${YELLOW}Warning:${NC} $1"
}

error() {
    echo -e "${RED}Error:${NC} $1" >&2
}

die() {
    error "$1"
    exit 1
}

# Print usage
usage() {
    cat <<EOF
Folio Installation Script

USAGE:
    install.sh [OPTIONS]

OPTIONS:
    -h, --help          Show this help message
    -y, --yes           Skip confirmation prompts
    --no-modify-path    Don't modify PATH in shell profile
    --no-config-init    Skip running 'folio config --init' after install
    --install-dir DIR   Install binary to DIR (default: ~/.local/bin)
    --version VERSION   Install specific version/branch (default: main)
    --uninstall         Remove folio installation

ENVIRONMENT VARIABLES:
    FOLIO_INSTALL_DIR   Override install directory
    FOLIO_BUILD_DIR     Override temporary build directory
    FOLIO_VERSION       Override version to install

EXAMPLES:
    # Standard installation
    curl -sSf https://raw.githubusercontent.com/kevinreber/folio/main/install.sh | sh

    # Non-interactive installation
    curl -sSf https://raw.githubusercontent.com/kevinreber/folio/main/install.sh | sh -s -- -y

    # Install to custom directory
    curl -sSf https://raw.githubusercontent.com/kevinreber/folio/main/install.sh | sh -s -- --install-dir /usr/local/bin

EOF
}

# Parse command line arguments
SKIP_CONFIRM=false
MODIFY_PATH=true
RUN_CONFIG_INIT=true
UNINSTALL=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -y|--yes)
            SKIP_CONFIRM=true
            shift
            ;;
        --no-modify-path)
            MODIFY_PATH=false
            shift
            ;;
        --no-config-init)
            RUN_CONFIG_INIT=false
            shift
            ;;
        --install-dir)
            FOLIO_INSTALL_DIR="$2"
            shift 2
            ;;
        --version)
            FOLIO_VERSION="$2"
            shift 2
            ;;
        --uninstall)
            UNINSTALL=true
            shift
            ;;
        *)
            die "Unknown option: $1. Use --help for usage."
            ;;
    esac
done

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)     OS_TYPE="linux" ;;
        Darwin*)    OS_TYPE="macos" ;;
        MINGW*|MSYS*|CYGWIN*) OS_TYPE="windows" ;;
        *)          die "Unsupported operating system: $OS" ;;
    esac

    case "$ARCH" in
        x86_64|amd64)   ARCH_TYPE="x86_64" ;;
        arm64|aarch64)  ARCH_TYPE="aarch64" ;;
        *)              die "Unsupported architecture: $ARCH" ;;
    esac

    info "Detected platform: $OS_TYPE ($ARCH_TYPE)"
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check for required dependencies
check_dependencies() {
    local missing=()

    if ! command_exists git; then
        missing+=("git")
    fi

    if ! command_exists curl && ! command_exists wget; then
        missing+=("curl or wget")
    fi

    if [ ${#missing[@]} -ne 0 ]; then
        die "Missing required dependencies: ${missing[*]}. Please install them first."
    fi
}

# Check for Rust installation
check_rust() {
    if command_exists rustc && command_exists cargo; then
        RUST_VERSION=$(rustc --version | cut -d ' ' -f 2)
        info "Found Rust $RUST_VERSION"
        return 0
    fi
    return 1
}

# Install Rust via rustup
install_rust() {
    info "Rust not found. Installing via rustup..."

    if command_exists curl; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    elif command_exists wget; then
        wget -qO- https://sh.rustup.rs | sh -s -- -y
    else
        die "Neither curl nor wget found. Cannot install Rust."
    fi

    # Source cargo environment
    if [ -f "$HOME/.cargo/env" ]; then
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    fi

    if ! check_rust; then
        die "Rust installation failed. Please install manually from https://rustup.rs/"
    fi

    success "Rust installed successfully"
}

# Clone and build folio
build_folio() {
    info "Building folio from source..."

    # Clean up any previous build
    rm -rf "$FOLIO_BUILD_DIR"
    mkdir -p "$FOLIO_BUILD_DIR"

    # Clone the repository
    info "Cloning repository..."
    git clone --depth 1 --branch "$FOLIO_VERSION" "$FOLIO_REPO" "$FOLIO_BUILD_DIR" 2>/dev/null || \
        git clone --depth 1 "$FOLIO_REPO" "$FOLIO_BUILD_DIR"

    # Build release binary
    info "Compiling (this may take a few minutes)..."
    cd "$FOLIO_BUILD_DIR"
    cargo build --release --quiet

    success "Build completed successfully"
}

# Install the binary
install_binary() {
    info "Installing folio to $FOLIO_INSTALL_DIR..."

    # Create install directory if it doesn't exist
    mkdir -p "$FOLIO_INSTALL_DIR"

    # Copy binary
    cp "$FOLIO_BUILD_DIR/target/release/folio" "$FOLIO_INSTALL_DIR/folio"
    chmod +x "$FOLIO_INSTALL_DIR/folio"

    success "Binary installed to $FOLIO_INSTALL_DIR/folio"
}

# Add to PATH if needed
setup_path() {
    if [ "$MODIFY_PATH" = false ]; then
        return
    fi

    # Check if install dir is already in PATH
    if echo "$PATH" | grep -q "$FOLIO_INSTALL_DIR"; then
        return
    fi

    info "Adding $FOLIO_INSTALL_DIR to PATH..."

    # Detect shell and profile file
    SHELL_NAME=$(basename "$SHELL")
    case "$SHELL_NAME" in
        bash)
            if [ -f "$HOME/.bash_profile" ]; then
                PROFILE="$HOME/.bash_profile"
            else
                PROFILE="$HOME/.bashrc"
            fi
            ;;
        zsh)
            PROFILE="$HOME/.zshrc"
            ;;
        fish)
            PROFILE="$HOME/.config/fish/config.fish"
            ;;
        *)
            PROFILE="$HOME/.profile"
            ;;
    esac

    # Add to profile if not already there
    PATH_LINE="export PATH=\"\$PATH:$FOLIO_INSTALL_DIR\""
    if [ -f "$PROFILE" ] && grep -q "$FOLIO_INSTALL_DIR" "$PROFILE"; then
        info "PATH already configured in $PROFILE"
    else
        echo "" >> "$PROFILE"
        echo "# Added by Folio installer" >> "$PROFILE"
        echo "$PATH_LINE" >> "$PROFILE"
        success "Added to $PROFILE"
        warn "Run 'source $PROFILE' or restart your terminal to update PATH"
    fi
}

# Run initial configuration
run_config_init() {
    if [ "$RUN_CONFIG_INIT" = false ]; then
        return
    fi

    if [ "$SKIP_CONFIRM" = true ]; then
        return
    fi

    echo ""
    read -p "Would you like to run initial configuration now? [Y/n] " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
        info "Running initial configuration..."
        # Add to PATH temporarily for this session
        export PATH="$PATH:$FOLIO_INSTALL_DIR"
        "$FOLIO_INSTALL_DIR/folio" config --init || true
    fi
}

# Cleanup build directory
cleanup() {
    info "Cleaning up..."
    rm -rf "$FOLIO_BUILD_DIR"
}

# Uninstall folio
uninstall() {
    info "Uninstalling folio..."

    # Remove binary
    if [ -f "$FOLIO_INSTALL_DIR/folio" ]; then
        rm -f "$FOLIO_INSTALL_DIR/folio"
        success "Removed $FOLIO_INSTALL_DIR/folio"
    else
        warn "Binary not found at $FOLIO_INSTALL_DIR/folio"
    fi

    echo ""
    echo "Note: Configuration and data at ~/.folio/ were NOT removed."
    echo "To completely remove folio data, run: rm -rf ~/.folio"
    echo ""
    echo "You may also want to remove the PATH entry from your shell profile."

    exit 0
}

# Print installation summary
print_summary() {
    echo ""
    echo "=============================================="
    success "Folio installed successfully!"
    echo "=============================================="
    echo ""
    echo "  Binary location: $FOLIO_INSTALL_DIR/folio"
    echo "  Data directory:  ~/.folio/"
    echo "  Config file:     ~/.folio/config.toml"
    echo ""
    echo "Quick start:"
    echo "  folio --help              # Show all commands"
    echo "  folio config --init       # Configure integrations"
    echo "  folio capture \"My win\"    # Capture an accomplishment"
    echo "  folio list                # View your activities"
    echo ""
    echo "Documentation: https://kevinreber.github.io/folio/"
    echo ""
}

# Confirmation prompt
confirm_install() {
    if [ "$SKIP_CONFIRM" = true ]; then
        return
    fi

    echo ""
    echo "This will install folio to: $FOLIO_INSTALL_DIR"
    echo ""
    read -p "Continue? [Y/n] " -n 1 -r
    echo ""

    if [[ ! $REPLY =~ ^[Yy]$ ]] && [[ -n $REPLY ]]; then
        echo "Installation cancelled."
        exit 0
    fi
}

# Main installation flow
main() {
    echo ""
    echo "  ╔═══════════════════════════════════════╗"
    echo "  ║     Folio Installation Script         ║"
    echo "  ║   Career Accomplishment Tracker       ║"
    echo "  ╚═══════════════════════════════════════╝"
    echo ""

    # Handle uninstall
    if [ "$UNINSTALL" = true ]; then
        uninstall
    fi

    # Detect platform
    detect_platform

    # Check dependencies
    check_dependencies

    # Confirm installation
    confirm_install

    # Check/install Rust
    if ! check_rust; then
        install_rust
    fi

    # Build folio
    build_folio

    # Install binary
    install_binary

    # Setup PATH
    setup_path

    # Cleanup
    cleanup

    # Run config init
    run_config_init

    # Print summary
    print_summary
}

# Run main
main
