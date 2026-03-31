# TermuxClaw

**Your personal AI bot running 24/7 on your Android phone.**

TermuxClaw is a Termux-only fork optimized for ARM64/ARMv7 Android devices. No desktop, no Docker, no cloud — just your phone.

---

## Quick Install (No Compilation!)

```bash
# 1. Install required apps from F-Droid:
#    - Termux
#    - Termux:API  
#    - Termux:Boot (for auto-start)

# 2. Open Termux and run:
pkg update -y && pkg install -y git
git clone https://github.com/foxy1402/termuxclaw.git
cd termuxclaw
chmod +x install.sh
./install.sh

# 3. Configure your bot:
zeroclaw onboard
```

**That's it!** The installer downloads a pre-built binary (~15-25MB) — no 20-minute compilation needed.

---

## What You Need

| Requirement | Details |
|-------------|---------|
| **Phone** | Android with ARM64 or ARMv7 CPU |
| **Storage** | 2GB free (pre-built) or 6GB (building from source) |
| **RAM** | 2GB+ recommended |
| **Apps** | Termux, Termux:API, Termux:Boot (all from F-Droid) |

> ⚠️ **Important**: Get Termux from [F-Droid](https://f-droid.org/packages/com.termux/), NOT Google Play. The Play Store version is outdated.

---

## Installation Steps

### Step 1: Install Android Apps

Download from F-Droid and **open each app once** after installing:

1. **[Termux](https://f-droid.org/packages/com.termux/)** — Terminal emulator
2. **[Termux:API](https://f-droid.org/packages/com.termux.api/)** — Phone hardware access (camera, GPS, SMS, etc.)
3. **[Termux:Boot](https://f-droid.org/packages/com.termux.boot/)** — Auto-start on device boot

### Step 2: Run the Installer

Open Termux and run:

```bash
pkg update -y && pkg install -y git
git clone https://github.com/foxy1402/termuxclaw.git
cd termuxclaw
chmod +x install.sh
./install.sh
```

**What the installer does:**
1. ✅ Checks your phone architecture (ARM64/ARMv7)
2. ✅ Sets up Termux storage access
3. ✅ Installs termux-api package
4. ✅ **Downloads pre-built binary** from GitHub releases (fast!)
5. ✅ Falls back to building from source if no binary available
6. ✅ Prompts to run configuration wizard
7. ✅ Offers to set up 24/7 auto-start with watchdog

### Step 3: Configure Your Bot

When the installer prompts "Run configuration wizard now?", say **yes** (or run later):

```bash
zeroclaw onboard
```

The wizard will guide you through:
- **AI Provider**: OpenAI, Anthropic, Gemini, local models, etc.
- **API Key**: Your provider's API key
- **Telegram Bot**: Your bot token (from @BotFather)
- **Autonomy Level**: full / supervised / readonly

### Step 4: Start Your Bot

```bash
# Test in foreground first:
zeroclaw daemon

# Stop with Ctrl+C when satisfied
```

---

## Running 24/7 (Auto-Start + Watchdog)

The installer can set this up automatically, or do it manually:

### Option A: Let Installer Set It Up

During `./install.sh`, when asked "Set up auto-start with watchdog?", say **yes**.

This installs:
- `~/.zeroclaw/watchdog.sh` — Monitors and auto-restarts zeroclaw if it crashes
- `~/.termux/boot/zeroclaw.sh` — Starts watchdog on device boot

### Option B: Manual Setup

```bash
# Copy watchdog script
mkdir -p ~/.zeroclaw
cp dev/zeroclaw-watchdog.sh ~/.zeroclaw/watchdog.sh
chmod +x ~/.zeroclaw/watchdog.sh

# Create Termux:Boot script
mkdir -p ~/.termux/boot
cat > ~/.termux/boot/zeroclaw.sh << 'EOF'
#!/data/data/com.termux/files/usr/bin/sh
mkdir -p ~/.zeroclaw/logs
exec ~/.zeroclaw/watchdog.sh >> ~/.zeroclaw/logs/boot.log 2>&1 &
EOF
chmod +x ~/.termux/boot/zeroclaw.sh
```

### Starting the Watchdog

```bash
# Start now (runs in background)
nohup ~/.zeroclaw/watchdog.sh &

# Or reboot your phone to test auto-start
```

### Managing the Watchdog

```bash
# View live logs
tail -f ~/.zeroclaw/logs/watchdog.log

# View crash history
cat ~/.zeroclaw/logs/crashes.log

# Stop the watchdog
pkill -f 'zeroclaw-watchdog'

# Check if running
pgrep -f 'zeroclaw-watchdog' && echo "Running" || echo "Not running"
```

### Android Battery Settings (Critical!)

To prevent Android from killing Termux:

1. **Settings → Apps → Termux → Battery**
   - Set to **Unrestricted**
   
2. **Settings → Apps → Termux:Boot → Battery**
   - Set to **Unrestricted**

3. **Disable battery optimization** for both apps

4. Some phones (Xiaomi, Samsung, Huawei) have extra settings:
   - Allow "Auto-start"
   - Disable "Adaptive battery"
   - Lock app in recent apps

---

## How the 24/7 System Works

```
┌─────────────────────────────────────────────────────────┐
│                    Phone Boots                          │
└─────────────────────┬───────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────┐
│  Termux:Boot runs ~/.termux/boot/zeroclaw.sh            │
└─────────────────────┬───────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────┐
│  Watchdog starts (~/.zeroclaw/watchdog.sh)              │
│  - Acquires wake lock                                   │
│  - Monitors zeroclaw daemon                             │
│  - Auto-restarts on crash (max 10/hour)                 │
└─────────────────────┬───────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────┐
│  ZeroClaw daemon runs                                   │
│  - Connects to Telegram/Discord/etc.                    │
│  - Processes messages                                   │
│  - Runs tools (camera, GPS, shell, etc.)                │
└─────────────────────────────────────────────────────────┘
```

**Crash Recovery Layers:**
1. **Layer 1 (Built-in)**: ZeroClaw's internal supervisor restarts crashed components
2. **Layer 2 (Watchdog)**: External script restarts crashed daemon process
3. **Layer 3 (Termux:Boot)**: Restarts everything after device reboot

---

## Installer Options

```bash
# Standard install (downloads pre-built binary - FAST!)
./install.sh

# Force build from source (if you want custom features)
./install.sh --force-build

# Minimal build for low-end phones (32-bit ARM, low RAM)
./install.sh --force-build --features termux-minimal

# Skip the onboard prompt
./install.sh --skip-onboard

# Skip build/download entirely (just setup)
./install.sh --skip-build

# Show help
./install.sh --help
```

---

## Useful Commands

```bash
# Configuration
zeroclaw onboard          # Run setup wizard
zeroclaw status           # Show current status
zeroclaw doctor           # Diagnose issues

# Running
zeroclaw daemon           # Run in foreground
zeroclaw chat             # Interactive chat in terminal
zeroclaw gateway          # Start web UI (localhost:8080)

# One-off queries
zeroclaw agent -m "What's my battery level?"
```

---

## Updating TermuxClaw

```bash
cd ~/termuxclaw
git pull
./install.sh
```

The installer will download the latest pre-built binary.

### Fresh Reinstall

```bash
# Remove config (keeps logs)
rm -f ~/.zeroclaw/config.toml

# Full wipe
rm -rf ~/.zeroclaw

# Then reinstall
cd ~/termuxclaw
./install.sh
zeroclaw onboard
```

---

## Troubleshooting

### `termux-* command not found`

```bash
pkg install -y termux-api
```
Also ensure the **Termux:API app** is installed from F-Droid.

### `zeroclaw: command not found`

The binary should be at `$PREFIX/bin/zeroclaw`. Try:
```bash
ls -la $PREFIX/bin/zeroclaw
# If missing, reinstall:
./install.sh
```

### Termux API commands hang

ZeroClaw has built-in timeout handling (5-30 seconds depending on command). If API hangs persist:
```bash
# Restart Termux:API
am force-stop com.termux.api
termux-vibrate -d 1  # Wake up the service
```

See [TERMUX-API-HANG-FIX.md](TERMUX-API-HANG-FIX.md) for details.

### Bot not auto-starting

1. Check boot script exists: `ls -la ~/.termux/boot/`
2. Check watchdog exists: `ls -la ~/.zeroclaw/watchdog.sh`
3. Open Termux:Boot app once (activates boot scripts)
4. Check Android battery settings (see above)
5. Reboot and wait 60 seconds

### Build fails (out of memory)

Use pre-built binary instead:
```bash
./install.sh  # Downloads binary, doesn't compile
```

Or use minimal features:
```bash
./install.sh --force-build --features termux-minimal
```

---

## Documentation

- [SELF-HEALING-GUIDE.md](SELF-HEALING-GUIDE.md) — Watchdog and 24/7 operation
- [TERMUX-API-HANG-FIX.md](TERMUX-API-HANG-FIX.md) — Handling API timeouts
- [CODEBASE_INDEX.md](CODEBASE_INDEX.md) — Project structure for developers
- [docs/setup-guides/termux-setup.md](docs/setup-guides/termux-setup.md) — Detailed setup guide

---

## Links

- **Repository**: https://github.com/foxy1402/termuxclaw
- **Releases**: https://github.com/foxy1402/termuxclaw/releases
- **Issues**: https://github.com/foxy1402/termuxclaw/issues
