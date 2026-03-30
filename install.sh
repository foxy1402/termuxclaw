#!/data/data/com.termux/files/usr/bin/bash
# ZeroClaw Termux (Android) Installer
# Optimized for ARM64/ARMv7 Android devices running Termux
set -euo pipefail

# --- Color and styling ---
if [[ -t 1 ]]; then
  BLUE='\033[0;34m'
  BOLD_BLUE='\033[1;34m'
  GREEN='\033[0;32m'
  YELLOW='\033[0;33m'
  RED='\033[0;31m'
  BOLD='\033[1m'
  DIM='\033[2m'
  RESET='\033[0m'
else
  BLUE='' BOLD_BLUE='' GREEN='' YELLOW='' RED='' BOLD='' DIM='' RESET=''
fi

CRAB="🦀"

info() {
  echo -e "${BLUE}${CRAB}${RESET} ${BOLD}$*${RESET}"
}

step_ok() {
  echo -e "  ${GREEN}✓${RESET} $*"
}

step_dot() {
  echo -e "  ${DIM}·${RESET} $*"
}

step_fail() {
  echo -e "  ${RED}✗${RESET} $*"
}

warn() {
  echo -e "${YELLOW}!${RESET} $*" >&2
}

error() {
  echo -e "${RED}✗${RESET} ${RED}$*${RESET}" >&2
}

have_cmd() {
  command -v "$1" >/dev/null 2>&1
}

is_termux() {
  [[ -n "${TERMUX_VERSION:-}" ]] || [[ -d "/data/data/com.termux" ]]
}

usage() {
  cat <<'USAGE'
ZeroClaw Termux (Android) Installer

Usage:
  ./install.sh [options]

Requirements:
  - Termux on Android (ARM64 or ARMv7)
  - At least 2GB RAM recommended
  - At least 6GB free storage
  - Internet connection

Options:
  --api-key <key>            API key (skips interactive prompt)
  --provider <id>            Provider (default: openrouter)
  --model <id>               Model (optional)
  --skip-onboard             Skip provider/API key configuration
  --install-rust             Install Rust via rustup if missing
  --skip-build               Skip build step (install only)
  --features <list>          Cargo features (default: termux)
  -h, --help                 Show help

Examples:
  # Interactive install (recommended for first time)
  ./install.sh

  # Non-interactive with API key
  ./install.sh --api-key "sk-..." --provider anthropic

  # Minimal build (32-bit ARM or low storage)
  ./install.sh --features termux-minimal

Environment Variables:
  ZEROCLAW_API_KEY           API key (used when --api-key not provided)
  ZEROCLAW_PROVIDER          Provider (default: openrouter)
  ZEROCLAW_MODEL             Model (optional)

Build Features:
  termux          - Standard build (ARM64, includes Prometheus metrics)
  termux-minimal  - Minimal build (ARM32 or storage-constrained)

USAGE
}

# --- Termux environment checks ---
check_termux_only() {
  if ! is_termux; then
    error "This installer requires Termux on Android."
    error ""
    error "ZeroClaw is now Termux-only (not cross-platform)."
    error ""
    error "To install:"
    error "  1. Download Termux from F-Droid (recommended)"
    error "     https://f-droid.org/en/packages/com.termux/"
    error "  2. Open Termux and run: pkg install git"
    error "  3. Clone: git clone https://github.com/foxy1402/termuxclaw"
    error "  4. Install: cd termuxclaw && ./install.sh"
    exit 1
  fi
}

check_architecture() {
  local arch
  arch="$(uname -m)"
  
  case "$arch" in
    aarch64|arm64)
      ARCH="aarch64"
      RUST_TARGET="aarch64-linux-android"
      DEFAULT_FEATURES="termux"
      step_ok "Architecture: ARM64 ($arch)"
      ;;
    armv7l)
      ARCH="armv7"
      RUST_TARGET="armv7-linux-androideabi"
      DEFAULT_FEATURES="termux-minimal"
      warn "Architecture: ARMv7 (32-bit) - using minimal build"
      warn "Prometheus metrics disabled (requires 64-bit)"
      ;;
    *)
      error "Unsupported architecture: $arch"
      error "ZeroClaw requires ARM64 (aarch64) or ARMv7 on Android"
      exit 1
      ;;
  esac
}

check_storage() {
  info "Checking Termux storage setup..."
  
  if [[ ! -d "$HOME/storage" ]]; then
    warn "Termux storage not set up."
    warn "This allows ZeroClaw to access your phone's files."
    
    if prompt_yes_no "Run termux-setup-storage now?"; then
      termux-setup-storage || {
        warn "Storage setup failed. You may need to grant permissions manually."
        warn "ZeroClaw will still work but won't access phone storage."
      }
      step_ok "Storage configured"
    else
      step_dot "Skipping storage setup (you can run 'termux-setup-storage' later)"
    fi
  else
    step_ok "Storage already configured"
  fi
}

check_termux_api() {
  if have_cmd termux-battery-status; then
    step_ok "termux-api installed"
    return 0
  fi
  
  warn "termux-api not installed"
  warn "Device features won't work: camera, GPS, notifications, SMS, etc."
  
  if prompt_yes_no "Install termux-api package? (Recommended)"; then
    pkg install -y termux-api
    warn "Note: Also install 'Termux:API' app from F-Droid for full functionality"
    step_ok "termux-api package installed"
  else
    step_dot "Skipping termux-api (limited device access)"
  fi
}

check_resources() {
  info "Checking system resources..."
  
  local total_ram_mb available_disk_mb
  
  # RAM check
  if [[ -r /proc/meminfo ]]; then
    total_ram_mb=$(awk '/MemTotal:/ {printf "%d\n", $2 / 1024}' /proc/meminfo)
    if [[ "$total_ram_mb" -lt 2048 ]]; then
      warn "Low RAM detected: ${total_ram_mb}MB (recommended: 2GB+)"
      warn "Build may be slow or fail. Consider using --features termux-minimal"
    else
      step_ok "RAM: ${total_ram_mb}MB"
    fi
  fi
  
  # Disk space check
  available_disk_mb=$(df -Pm "$HOME" | awk 'NR==2 {print $4}')
  if [[ "$available_disk_mb" -lt 6144 ]]; then
    warn "Low disk space: ${available_disk_mb}MB free (recommended: 6GB+)"
    warn "Build artifacts are large. Free up space if build fails."
  else
    step_ok "Free disk: ${available_disk_mb}MB"
  fi
  
  # Battery warning
  if have_cmd termux-battery-status; then
    local battery_percent
    battery_percent=$(termux-battery-status | grep -oP '(?<="percentage": )[0-9]+' || echo "0")
    if [[ "$battery_percent" -lt 30 ]]; then
      warn "Low battery: ${battery_percent}%"
      warn "Plug in your device before building (build takes 5-15 minutes)"
    fi
  fi
}

prompt_yes_no() {
  local prompt="$1"
  local response
  
  while true; do
    read -rp "  ${prompt} [y/n]: " response
    case "${response,,}" in
      y|yes) return 0 ;;
      n|no) return 1 ;;
      *) echo "  Please answer y or n" ;;
    esac
  done
}

# --- Package management ---
install_dependencies() {
  info "Installing build dependencies..."
  
  pkg update -y -q || {
    error "Failed to update package lists"
    exit 1
  }
  
  local packages=()
  
  # Essential build tools
  have_cmd git || packages+=(git)
  have_cmd clang || packages+=(clang)
  have_cmd make || packages+=(make)
  have_cmd pkg-config || packages+=(pkg-config)
  
  # Rust toolchain (optional, can install via rustup)
  if [[ "${INSTALL_RUST:-0}" == "1" ]] && ! have_cmd rustc; then
    packages+=(rust)
  fi
  
  if [[ ${#packages[@]} -gt 0 ]]; then
    step_dot "Installing: ${packages[*]}"
    pkg install -y "${packages[@]}" || {
      error "Failed to install dependencies: ${packages[*]}"
      exit 1
    }
    step_ok "Dependencies installed"
  else
    step_ok "All dependencies already installed"
  fi
}

install_rust() {
  if have_cmd rustc && have_cmd cargo; then
    local rust_version
    rust_version=$(rustc --version | awk '{print $2}')
    step_ok "Rust already installed: $rust_version"
    return 0
  fi
  
  info "Installing Rust toolchain via rustup..."
  
  if ! have_cmd rustup; then
    step_dot "Downloading rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    
    # Source cargo env
    if [[ -f "$HOME/.cargo/env" ]]; then
      # shellcheck source=/dev/null
      source "$HOME/.cargo/env"
    fi
  fi
  
  if have_cmd rustup; then
    rustup default stable
    rustup target add "$RUST_TARGET"
    step_ok "Rust installed successfully"
  else
    error "Failed to install Rust. Install manually: pkg install rust"
    exit 1
  fi
}

# --- Build process ---
build_zeroclaw() {
  local features="${CARGO_FEATURES:-$DEFAULT_FEATURES}"
  
  info "Building ZeroClaw (this may take 5-15 minutes)..."
  step_dot "Features: $features"
  step_dot "Target: $RUST_TARGET"
  
  warn "Keep Termux in foreground or use a wake lock to prevent interruption"
  
  if ! cargo build --release --no-default-features --features "$features" --target "$RUST_TARGET"; then
    error "Build failed"
    error ""
    error "Common issues:"
    error "  - Low RAM: Try --features termux-minimal"
    error "  - Low disk space: Free up at least 6GB"
    error "  - Internet connection lost during dependency download"
    error "  - Battery died during build (plug in your device!)"
    exit 1
  fi
  
  step_ok "Build completed successfully"
}

install_binary() {
  local binary_path="target/$RUST_TARGET/release/zeroclaw"
  
  if [[ ! -x "$binary_path" ]]; then
    error "Binary not found at $binary_path"
    exit 1
  fi
  
  info "Installing zeroclaw to \$PREFIX/bin..."
  
  local prefix="${PREFIX:-/data/data/com.termux/files/usr}"
  local bin_dir="$prefix/bin"
  
  if [[ ! -d "$bin_dir" ]]; then
    error "Termux bin directory not found: $bin_dir"
    exit 1
  fi
  
  install -m 0755 "$binary_path" "$bin_dir/zeroclaw" || {
    error "Failed to install binary to $bin_dir"
    exit 1
  }
  
  step_ok "Installed to $bin_dir/zeroclaw"
  
  # Verify installation
  if zeroclaw --version >/dev/null 2>&1; then
    local version
    version=$(zeroclaw --version)
    step_ok "Installation verified: $version"
  else
    warn "Installation complete but 'zeroclaw --version' failed"
  fi
}

# --- Configuration ---
configure_provider() {
  if [[ "${SKIP_ONBOARD:-0}" == "1" ]]; then
    step_dot "Skipping provider configuration (--skip-onboard)"
    return 0
  fi
  
  info "Configuring AI provider..."
  
  local provider="${ZEROCLAW_PROVIDER:-${PROVIDER:-openrouter}}"
  local api_key="${ZEROCLAW_API_KEY:-${API_KEY:-}}"
  local model="${ZEROCLAW_MODEL:-${MODEL:-}}"
  
  # Interactive prompts if not provided
  if [[ -z "$api_key" ]]; then
    echo ""
    echo "  You need an API key from an AI provider."
    echo "  Recommended: OpenRouter (access to 200+ models with one key)"
    echo "  Get a key: https://openrouter.ai/keys"
    echo ""
    read -rp "  Enter your API key: " api_key
  fi
  
  if [[ -z "$api_key" ]]; then
    warn "No API key provided. You'll need to configure manually later."
    step_dot "Run: zeroclaw onboard"
    return 0
  fi
  
  # Run onboard command
  if ZEROCLAW_API_KEY="$api_key" ZEROCLAW_PROVIDER="$provider" zeroclaw onboard --non-interactive; then
    step_ok "Provider configured: $provider"
    if [[ -n "$model" ]]; then
      step_dot "Model: $model"
    fi
  else
    warn "Configuration failed. Run manually: zeroclaw onboard"
  fi
}

setup_termux_boot() {
  info "Termux:Boot auto-start & watchdog setup"
  
  echo ""
  echo "  ZeroClaw can auto-start when your Android device boots with built-in crash recovery."
  echo "  This requires the Termux:Boot app from F-Droid."
  echo ""
  echo "  ${BOLD}Features:${RESET}"
  echo "    • Automatic startup on device boot"
  echo "    • Self-healing: auto-restarts if zeroclaw crashes"
  echo "    • Runs 24/7 without manual intervention"
  echo "    • Health monitoring and crash logging"
  echo ""
  
  if ! prompt_yes_no "Set up auto-start with watchdog?"; then
    step_dot "Skipping auto-start setup"
    return 0
  fi
  
  # Install watchdog script
  local zeroclaw_dir="$HOME/.zeroclaw"
  local watchdog_script="$zeroclaw_dir/watchdog.sh"
  mkdir -p "$zeroclaw_dir"
  
  step_dot "Installing watchdog script to ~/.zeroclaw/watchdog.sh"
  
  # Copy watchdog from dev/ to ~/.zeroclaw/
  if [[ -f "dev/zeroclaw-watchdog.sh" ]]; then
    cp dev/zeroclaw-watchdog.sh "$watchdog_script"
    chmod +x "$watchdog_script"
    step_ok "Watchdog installed"
  else
    warn "Watchdog script not found in dev/zeroclaw-watchdog.sh"
    warn "You can download it from: https://github.com/foxy1402/termuxclaw/tree/master/dev"
    return 1
  fi
  
  # Set up Termux:Boot script
  local boot_dir="$HOME/.termux/boot"
  local boot_script="$boot_dir/zeroclaw.sh"
  
  mkdir -p "$boot_dir"
  
  step_dot "Creating Termux:Boot script at ~/.termux/boot/zeroclaw.sh"
  
  cat > "$boot_script" <<'BOOTSCRIPT'
#!/data/data/com.termux/files/usr/bin/sh
#
# ZeroClaw auto-start script with watchdog
# Runs on device boot via Termux:Boot
#

# Ensure log directory exists
mkdir -p ~/.zeroclaw/logs

# Start the watchdog in the background
# The watchdog will start zeroclaw daemon and keep it running 24/7
exec ~/.zeroclaw/watchdog.sh >> ~/.zeroclaw/logs/boot.log 2>&1 &
BOOTSCRIPT
  
  chmod +x "$boot_script"
  step_ok "Termux:Boot script created"
  
  echo ""
  info "Setup complete!"
  echo ""
  echo "  ${BOLD}Next steps:${RESET}"
  echo "    1. Install Termux:Boot from F-Droid:"
  echo "       https://f-droid.org/packages/com.termux.boot/"
  echo ""
  echo "    2. Open Termux:Boot app once (to enable boot scripts)"
  echo ""
  echo "    3. Reboot your device to test auto-start"
  echo ""
  echo "  ${BOLD}Manual control:${RESET}"
  echo "    Start now:   nohup ~/.zeroclaw/watchdog.sh &"
  echo "    Stop:        pkill -f 'zeroclaw-watchdog'"
  echo "    View logs:   tail -f ~/.zeroclaw/logs/watchdog.log"
  echo "    Crash log:   tail -f ~/.zeroclaw/logs/crashes.log"
  echo ""
}
  
  chmod +x "$boot_script"
  
  step_ok "Boot script created: $boot_script"
  echo ""
  echo "  Next steps:"
  echo "    1. Install Termux:Boot from F-Droid"
  echo "       https://f-droid.org/en/packages/com.termux.boot/"
  echo "    2. Restart your device"
  echo "    3. ZeroClaw will auto-start on boot"
  echo ""
  echo "  To view logs: tail -f ~/.zeroclaw/daemon.log"
  echo "  To stop daemon: pkill -f 'zeroclaw daemon'"
  echo ""
}

# --- Main installer flow ---
main() {
  echo ""
  info "ZeroClaw Termux (Android) Installer"
  echo ""
  
  # Parse arguments
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --api-key)
        API_KEY="$2"
        shift 2
        ;;
      --provider)
        PROVIDER="$2"
        shift 2
        ;;
      --model)
        MODEL="$2"
        shift 2
        ;;
      --skip-onboard)
        SKIP_ONBOARD=1
        shift
        ;;
      --install-rust)
        INSTALL_RUST=1
        shift
        ;;
      --skip-build)
        SKIP_BUILD=1
        shift
        ;;
      --features)
        CARGO_FEATURES="$2"
        shift 2
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        error "Unknown option: $1"
        usage
        exit 1
        ;;
    esac
  done
  
  # Validate Termux environment
  check_termux_only
  check_architecture
  
  # Environment setup
  check_storage
  check_termux_api
  check_resources
  
  # Install dependencies
  install_dependencies
  
  # Install Rust if needed
  if [[ "${INSTALL_RUST:-0}" == "1" ]] || ! have_cmd cargo; then
    install_rust
  fi
  
  # Build ZeroClaw
  if [[ "${SKIP_BUILD:-0}" != "1" ]]; then
    build_zeroclaw
    install_binary
  else
    step_dot "Build skipped (--skip-build)"
  fi
  
  # Configure provider
  configure_provider
  
  # Optional: Termux:Boot setup
  setup_termux_boot
  
  # Success message
  echo ""
  info "Installation complete! 🎉"
  echo ""
  echo "  ${BOLD}Quick start:${RESET}"
  echo "    zeroclaw --help              Show all commands"
  echo "    zeroclaw gateway             Start web UI (http://localhost:8080)"
  echo "    zeroclaw chat                Interactive chat in terminal"
  echo ""
  echo "  ${BOLD}Documentation:${RESET}"
  echo "    docs/setup-guides/termux-setup.md"
  echo ""
  echo "  ${BOLD}Support:${RESET}"
  echo "    https://github.com/foxy1402/termuxclaw"
  echo ""
}

# Run main installer
main "$@"
