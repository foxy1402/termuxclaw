# Termux (Android) Setup Guide

This guide explains how to build and run ZeroClaw on Android using Termux.

## Overview

ZeroClaw can run natively on Android devices through Termux, providing a full-featured AI assistant that runs entirely on your phone. The Termux build is optimized for mobile constraints with reduced dependencies and smaller binary size.

## Prerequisites

1. **Termux** (install from F-Droid or Google Play)
2. **Rust toolchain** (installed via Termux)
3. **Basic packages**: `pkg install git clang`

Optional:
- **Termux:Boot** - For auto-starting ZeroClaw on device boot
- **Termux:API** - For accessing Android device features (camera, GPS, notifications, etc.)

## Installation

### Method 1: Build Natively on Android

Build ZeroClaw directly on your Android device:

```bash
# Install prerequisites
pkg install rust git clang

# Clone repository
git clone https://github.com/zeroclaw-labs/zeroclaw
cd zeroclaw

# Build with Termux-optimized features
cargo build --release --no-default-features --features termux

# Install to Termux prefix
cp target/release/zeroclaw $PREFIX/bin/
```

### Method 2: Cross-Compile from Linux Host

Cross-compile from a Linux machine for faster builds:

```bash
# Install Android NDK (r21+ required)
# Set NDK_HOME to your NDK installation directory

# Add Android target
rustup target add aarch64-linux-android

# Configure linker in .cargo/config.toml (already configured in repo)

# Cross-compile
export PATH="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"
cargo build --release \
    --target aarch64-linux-android \
    --no-default-features \
    --features termux

# Transfer binary to Android device
adb push target/aarch64-linux-android/release/zeroclaw /data/local/tmp/
# Then move to Termux: mv /data/local/tmp/zeroclaw $PREFIX/bin/
```

## Feature Flags

### `termux` (Recommended for 64-bit Android)

Includes:
- ✅ `channel-nostr` - Pure Rust Nostr p2p messaging
- ✅ `channel-lark` - Lightweight Lark/Feishu integration
- ✅ `skill-creation` - Autonomous skill building
- ✅ `observability-prometheus` - Metrics (requires 64-bit ARM)

Excludes:
- ❌ `rag-pdf` - C++ Poppler dependency (Android incompatible)
- ❌ `channel-matrix` - Heavy E2EE dependencies
- ❌ `memory-postgres` - libpq C library not available
- ❌ `browser-native` - No WebDriver on Android
- ❌ `sandbox-landlock`, `sandbox-bubblewrap` - Not supported on Android
- ❌ `voice-wake` - Audio capture requires root
- ❌ `plugins-wasm` - RAM-heavy on mobile
- ❌ `whatsapp-web` - Massive dependency tree

Build command:
```bash
cargo build --release --no-default-features --features termux
```

### `termux-minimal` (For 32-bit Android or Storage-Constrained Devices)

Same as `termux` but drops Prometheus metrics (requires 64-bit AtomicU64):

Build command:
```bash
cargo build --release --no-default-features --features termux-minimal
```

## Configuration

Create a minimal config at `~/.zeroclaw/config.toml`:

```toml
[agent]
default_model = "anthropic/claude-3-5-sonnet-20241022"

[anthropic]
api_key = "sk-ant-..."  # Or use environment variable ANTHROPIC_API_KEY

[channels.telegram]
bot_token = "123456:ABC-DEF..."
allowed_users = ["@yourusername"]

[security]
autonomy = "supervised"  # or "full" if you trust the agent completely
workspace_dir = "/data/data/com.termux/files/home/workspace"
```

## Auto-Start on Boot (Optional)

1. Install **Termux:Boot** from F-Droid or Google Play

2. Copy the boot script:
   ```bash
   mkdir -p ~/.termux/boot
   cp dev/termux-boot.sh ~/.termux/boot/zeroclaw.sh
   chmod +x ~/.termux/boot/zeroclaw.sh
   ```

3. Restart your device or run manually:
   ```bash
   ~/.termux/boot/zeroclaw.sh
   ```

4. View logs:
   ```bash
   tail -f ~/.zeroclaw/daemon.log
   ```

5. Stop the daemon:
   ```bash
   pkill -f "zeroclaw daemon"
   ```

## Termux-Specific Tools

ZeroClaw includes Android-specific tools when running on Termux:

### `termux_api` Tool

Requires `pkg install termux-api` and **Termux:API** app installed.

Available commands:
- `termux-battery-status` - Battery level and charging status
- `termux-location` - GPS coordinates
- `termux-camera-photo` - Capture photos (replaces screenshot tool on Android)
- `termux-sms-send` - Send SMS
- `termux-notification` - Show Android notifications
- `termux-toast` - Display toast messages
- `termux-vibrate` - Vibrate device
- `termux-wifi-connectioninfo` - WiFi network info
- And many more...

### Browser Opening

The `browser_open` tool automatically uses `termux-open-url` on Android instead of `xdg-open`.

Install: `pkg install termux-api`

### Screenshot Alternative

The `screenshot` tool doesn't work on Android (no X11). Use `termux_api` instead:

```bash
termux-camera-photo output.jpg
```

## Platform-Specific Limitations

Features unavailable on Termux/Android:

| Feature | Reason | Alternative |
|---------|--------|-------------|
| `screenshot` tool | No X11/display server | `termux-camera-photo` via `termux_api` |
| Service management (`zeroclaw service install`) | No systemd/launchd | Use Termux:Boot |
| PDF ingestion (`rag-pdf`) | C++ Poppler dependency | Disable feature |
| PostgreSQL memory backend | libpq not available | Use SQLite or Markdown |
| Browser automation | No WebDriver/Chromium | Use `web_fetch` tool |
| Voice wake detection | Requires ALSA/root | Disable feature |

## Storage Optimization

Android storage is limited. Remove unnecessary files:

```bash
# Delete CI/docs/benchmarks (not needed at runtime)
rm -rf .github/ docs/ benches/ tests/manual/ dev/ python/ scripts/

# Keep only essential files:
# - src/ (source code)
# - Cargo.toml, Cargo.lock
# - web/dist/ (pre-built web UI)
# - tool_descriptions/ (runtime tool metadata)
# - skills/ (skill definitions)
```

Pre-build the web UI on a desktop machine before transferring to Android (much faster than building on device):

```bash
# On desktop:
cd web
npm install
npm run build

# Then transfer entire zeroclaw/ directory to Android
```

## Troubleshooting

### Build fails with "linker not found"

Install clang: `pkg install clang`

### "xdg-open not found" error

Update to latest version with Termux detection fix, or disable `browser_open` tool in config.

### Out of memory during compilation

Use `termux-minimal` feature or increase swap:
```bash
# Create 2GB swap file
dd if=/dev/zero of=/data/local/tmp/swapfile bs=1M count=2048
chmod 600 /data/local/tmp/swapfile
mkswap /data/local/tmp/swapfile
swapon /data/local/tmp/swapfile
```

### Service commands not working

Service management is not supported on Android. Use Termux:Boot for auto-start (see above).

### Tools failing at runtime

Some tools detect Linux and try Linux-specific commands that don't exist in Termux. Report these as bugs - they should detect Termux via `$PREFIX` environment variable.

## Performance Notes

- **Binary size**: ~15-25 MB with `termux` features (vs ~40+ MB with all features)
- **RAM usage**: ~50-150 MB depending on active channels and memory backend
- **Battery**: Gateway/daemon mode uses ~2-5% battery per hour on idle

## Web Dashboard Access

Start the gateway on your Android device:

```bash
zeroclaw gateway --bind 127.0.0.1:8080
```

Then access from:
- **Same device**: Open browser to `http://127.0.0.1:8080`
- **LAN devices**: Use device IP, e.g. `http://192.168.1.42:8080`
- **Internet**: Use a tunnel (ngrok, cloudflared, tailscale)

The web UI is fully touch-optimized for Android browsers.

## Next Steps

- Configure channels (Telegram, Discord, Nostr, etc.)
- Set up memory backend (SQLite recommended for mobile)
- Install Termux:API for device integration
- Explore the web dashboard at `http://localhost:8080`
- Read [TERMUX-TRIM.md](../../termux-trim.md) for full optimization details

## Support

- **Issues**: https://github.com/zeroclaw-labs/zeroclaw/issues
- **Discussions**: https://github.com/zeroclaw-labs/zeroclaw/discussions
- **Docs**: https://github.com/zeroclaw-labs/zeroclaw/tree/master/docs
