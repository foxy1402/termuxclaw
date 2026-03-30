# install.sh Termux-Only Trim Guide

**Status**: install.sh still contains cross-platform code  
**Action needed**: Simplify to Termux-only

---

## Current State

The install.sh script (1803 lines) contains:
- **10+ Darwin/macOS references** - Hardware detection, Xcode CLI tools, memory detection
- **Multiple Linux distro handlers** - Alpine, Debian/Ubuntu, Fedora, Arch, etc.
- **Docker/container detection** - Container runtime checks
- **Cross-platform package managers** - apk, apt-get, dnf, pacman, brew

---

## Recommended Termux-Only Changes

###1. Remove macOS/Darwin Code

**Lines to remove/simplify**:
- Line 180-188: `Darwin` case in `get_total_memory_mb()`
- Line 242-246: `Darwin` targets in `detect_release_target()`
- Line 272-274: macOS desktop detection in `detect_device_class()`
- Line 630+: Darwin case in dependency installation
- Line 1283-1285: Xcode/CLT license check
- Line 1579+: macOS download URLs
- Line 1752+: Darwin cases in service management

**Replace with**:
```bash
# Termux-only: Always return empty/fail for macOS functions
get_total_memory_mb() {
  if [[ -r /proc/meminfo ]]; then
    awk '/MemTotal:/ {printf "%d\n", $2 / 1024}' /proc/meminfo
  fi
}

detect_release_target() {
  local arch
  arch="$(uname -m)"
  
  case "$arch" in
    aarch64|arm64)
      echo "aarch64-linux-android"
      ;;
    armv7l)
      echo "armv7-linux-androideabi"
      ;;
    *)
      error "Unsupported architecture: $arch. ZeroClaw requires ARM64 or ARMv7 on Android."
      exit 1
      ;;
  esac
}

detect_device_class() {
  echo "mobile"  # Always mobile on Termux
}
```

### 2. Simplify Package Manager Detection

**Current**: Checks apk, apt-get, dnf, yum, pacman, zypper, brew  
**Termux-only**: Only `pkg` (Termux's package manager)

```bash
install_system_deps() {
  if ! is_termux; then
    error "This installer only supports Termux (Android)."
    error "For desktop Linux, please build manually."
    exit 1
  fi
  
  info "Installing Termux build dependencies..."
  pkg update -y
  pkg install -y rust clang git binutils
}
```

### 3. Remove Container/Docker Code

**Lines to remove**:
- Line 14-18: `_is_container_runtime()`
- Line 27-37: pacman container special handling
- Line 256-259: Container detection in `detect_device_class()`
- All Docker-related flags and functions (`--docker`, Docker image building, etc.)

**Reason**: Docker doesn't run on standard Android/Termux

### 4. Remove Desktop Linux Distro Handlers

**Simplify to Termux-only**:
```bash
# Remove all these distro-specific checks:
- Alpine apk
- Debian/Ubuntu apt-get
- Fedora/RHEL dnf/yum
- Arch pacman  
- openSUSE zypper
- macOS brew

# Keep only:
is_termux() {
  [[ -n "${TERMUX_VERSION:-}" || -d "/data/data/com.termux" ]]
}

if ! is_termux; then
  error "ZeroClaw installer requires Termux on Android."
  error "Download Termux from F-Droid: https://f-droid.org/en/packages/com.termux/"
  exit 1
fi
```

### 5. Add Termux Early Exit Check

**Add at start of main()**:
```bash
main() {
  # Termux-only check (fail fast)
  if ! is_termux; then
    error "This version of ZeroClaw is Termux-only (Android)."
    error ""
    error "To install:"
    error "  1. Install Termux from F-Droid"
    error "  2. Run: pkg install git"
    error "  3. Clone: git clone https://github.com/foxy1402/termuxclaw"
    error "  4. Run: cd termuxclaw && ./install.sh"
    error ""
    error "Termux download: https://f-droid.org/en/packages/com.termux/"
    exit 1
  fi
  
  # Rest of installer...
}
```

### 6. Simplify Feature Detection

**Remove**:
- musl vs glibc detection (Android uses Bionic)
- Desktop vs embedded device classification
- Multi-distro kernel/systemd checks

**Keep**:
- ARM64 vs ARMv7 detection (for correct Rust target)
- Memory/disk space checks (still relevant on mobile)
- Termux $PREFIX detection

### 7. Update Usage/Help Text

```bash
usage() {
  cat <<'USAGE'
ZeroClaw installer — Termux (Android) one-click bootstrap

Usage:
  ./install.sh [options]

Requirements:
  - Termux on Android (ARM64 or ARMv7)
  - At least 2GB RAM
  - At least 6GB free storage

Options:
  --api-key <key>            API key (skips interactive prompt)
  --provider <id>            Provider (default: openrouter)
  --model <id>               Model (optional)
  --skip-onboard             Skip provider/API key configuration
  --install-rust             Install Rust via rustup if missing
  -h, --help                 Show help

Examples:
  # One-click install (interactive)
  ./install.sh

  # Non-interactive with API key
  ./install.sh --api-key "sk-..." --provider openrouter

  # Install Rust first
  ./install.sh --install-rust

Environment:
  ZEROCLAW_API_KEY           Used when --api-key is not provided
  ZEROCLAW_PROVIDER          Used when --provider is not provided (default: openrouter)
  ZEROCLAW_MODEL             Used when --model is not provided
USAGE
}
```

### 8. Termux-Specific Enhancements

**Add**:
- Automatic `termux-api` package installation prompt
- Termux:Boot setup wizard (interactive)
- Storage permission check (`termux-setup-storage`)
- Battery optimization warning
- Wake lock suggestion for long builds

```bash
check_termux_environment() {
  info "Checking Termux environment..."
  
  # Storage access
  if [[ ! -d "$HOME/storage" ]]; then
    warn "Storage not set up. Running termux-setup-storage..."
    termux-setup-storage || warn "Storage setup failed (you may need to grant permissions)"
  fi
  
  # Termux API (optional but recommended)
  if ! have_cmd termux-battery-status; then
    warn "termux-api not installed. Device features (camera, GPS, notifications) won't work."
    if prompt_yes_no "Install termux-api package?"; then
      pkg install -y termux-api
    fi
  fi
  
  # Battery optimization warning
  warn "For best results during build:"
  warn "  1. Plug in your device (builds can take 5-15 minutes)"
  warn "  2. Keep Termux in foreground or use a wake lock"
  warn "  3. Ensure you have stable internet connection"
}
```

---

## Estimated LOC Reduction

| Section | Before | After | Savings |
|---------|--------|-------|---------|
| OS detection | ~200 lines | ~30 lines | **~85%** |
| Package managers | ~300 lines | ~50 lines | **~83%** |
| Docker/containers | ~150 lines | 0 lines | **100%** |
| macOS-specific | ~100 lines | 0 lines | **100%** |
| Distro handlers | ~400 lines | ~50 lines | **~87%** |
| **Total** | **~1800 lines** | **~600 lines** | **~67%** |

---

## Implementation Strategy

Given the size (1803 lines), I recommend:

1. **Option A**: Create new `install-termux.sh` (clean rewrite, ~600 lines)
2. **Option B**: Keep `install.sh` as cross-platform backup, symlink `install-termux.sh` as main
3. **Option C**: Aggressive inline editing of current `install.sh` (risky, many dependencies)

**Recommended**: Option A - Create simplified `install-termux.sh`, rename old one to `install.cross-platform.sh.bak`

---

## Next Steps

1. Create simplified Termux-only installer (`install-termux.sh`)
2. Test on ARM64 Termux device
3. Test on ARMv7 (32-bit) device if available
4. Add Termux-Boot integration wizard
5. Update README.md to point to new installer

Would you like me to proceed with creating the simplified Termux-only installer?
