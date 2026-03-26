# TermuxClaw

TermuxClaw is a personal AI bot that runs directly on your Android phone with Termux.

This guide is written for non-technical users: copy commands exactly and follow step by step.

---

## What you need before starting

- Android phone
- Internet connection
- About 6-10 GB free storage
- 20-60 minutes for first setup

Install these Android apps first:

1. **Termux** (from F-Droid recommended)
2. **Termux:API**
3. **Termux:Boot** (for auto-start on phone reboot)

Important: open each app once after installing.

---

## Step 1) Open Termux and prepare packages

Run:

```bash
pkg update -y && pkg upgrade -y
pkg install -y git rust termux-api openssl-tool
termux-setup-storage
```

When Android asks for storage permission, press **Allow**.

---

## Step 2) Download TermuxClaw

Choose one method.

### Method A (Git clone, recommended)

```bash
git clone https://github.com/foxy1402/termuxclaw.git
cd termuxclaw
```

### Method B (wget zip download)

```bash
pkg install -y wget unzip
wget -O termuxclaw.zip https://github.com/foxy1402/termuxclaw/archive/refs/heads/master.zip
unzip termuxclaw.zip
cd termuxclaw-master
```

---

## Step 3) Install / build

### Easy auto installer (recommended)

```bash
bash install.sh --prefer-prebuilt
```

If prebuilt binary is available, it uses that. If not, it builds from source.

Termux installer update:

- It now syncs `zeroclaw` into Termux bin (`$PREFIX/bin`) so `zeroclaw` works immediately in the same shell.
- In interactive terminal mode, it launches the full `zeroclaw onboard` experience (arrow-key navigation, paste support, advanced provider fields like custom base URL), instead of a limited inline prompt flow.
- During onboarding/quick setup, you can now choose autonomy mode with arrow keys:
  - `Full (unrestricted)` — no policy limits
  - `Supervised (workspace-scoped)` — safer limits + approvals

### If you want prebuilt only (no compile fallback)

```bash
bash install.sh --prebuilt-only
```

### Manual build (advanced)

```bash
cargo build --release --locked
cargo install --path . --force --locked
```

---

## Step 4) First run setup

If you used `bash install.sh --prefer-prebuilt` in an interactive Termux session, onboarding is usually started automatically at the end of install.

Run onboarding:

```bash
zeroclaw onboard
```

Follow prompts:

- Choose your AI provider/model
- Add API key or login
- Configure channels/tools
- Save config

Quick check:

```bash
zeroclaw status
zeroclaw doctor
```

---

## Step 5) Run your bot

### Foreground (for testing)

```bash
zeroclaw daemon
```

Stop it with `Ctrl + C`.

### Background (keep running in Termux session)

```bash
nohup zeroclaw daemon > ~/.zeroclaw/daemon.log 2>&1 &
```

Check log:

```bash
tail -f ~/.zeroclaw/daemon.log
```

---

## Step 6) Make it run 24/7 (auto start after phone reboot)

This uses **Termux:Boot**.

### 6.1 Create boot script

Run:

```bash
mkdir -p ~/.termux/boot
cat > ~/.termux/boot/start-termuxclaw.sh << 'EOF'
#!/data/data/com.termux/files/usr/bin/bash
termux-wake-lock
sleep 10
nohup zeroclaw daemon >> "$HOME/.zeroclaw/daemon.log" 2>&1 &
EOF
chmod +x ~/.termux/boot/start-termuxclaw.sh
```

### 6.2 Android battery settings (very important)

For **Termux** and **Termux:Boot**:

- Battery usage: set to **Unrestricted**
- Disable battery optimization
- Allow background activity
- Allow auto-start (if your phone brand has this option)

### 6.3 Test reboot flow

1. Reboot phone
2. Wait 30-60 seconds
3. Open Termux
4. Run:

```bash
ps -ef | grep zeroclaw
tail -n 50 ~/.zeroclaw/daemon.log
```

If you see `zeroclaw daemon` process and fresh logs, auto-start works.

---

## Keep TermuxClaw alive better (24/7 stability tips)

- Keep at least 1-2 GB free RAM
- Keep at least 2 GB free storage
- Update packages weekly:

```bash
pkg update -y && pkg upgrade -y
```

- Restart bot if needed:

```bash
pkill -f "zeroclaw daemon"
nohup zeroclaw daemon > ~/.zeroclaw/daemon.log 2>&1 &
```

---

## Update TermuxClaw later

If installed with git clone:

```bash
cd ~/termuxclaw
git pull
bash install.sh --prefer-prebuilt
```

If installed with zip, re-download latest zip and repeat install steps.

### Fresh reinstall (reset old config and start as new)

If you want a completely fresh onboarding (new config defaults, clean workspace files), remove old runtime data first:

```bash
rm -f ~/.zeroclaw/config.toml
rm -rf ~/.zeroclaw/workspace
```

Then reinstall and onboard again:

```bash
cd ~/termuxclaw
bash install.sh --prefer-prebuilt
zeroclaw onboard
```

Optional full wipe (also removes logs/state/secrets cache under `~/.zeroclaw`):

```bash
rm -rf ~/.zeroclaw
```

---

## Common problems and quick fixes

### `termux-* command not found`

Run:

```bash
pkg install -y termux-api
```

Also make sure **Termux:API app** is installed.

### `zeroclaw: command not found` after install

With current installer versions this should be fixed on Termux (binary is synced to `$PREFIX/bin`).

If you installed using an older script, run:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Then re-run install:

```bash
bash install.sh --prefer-prebuilt
```

### Build fails / too slow

Try prebuilt-only:

```bash
bash install.sh --prebuilt-only
```

### Bot not starting on reboot

- Confirm `~/.termux/boot/start-termuxclaw.sh` exists and is executable
- Re-check battery optimization settings
- Open Termux:Boot app once after install/update

### Permission issues

Run:

```bash
termux-setup-storage
```

---

## Useful commands

```bash
zeroclaw onboard
zeroclaw status
zeroclaw doctor
zeroclaw daemon
zeroclaw agent
zeroclaw agent -m "hello"
```

---

## Official links for this fork

- Repository: `https://github.com/foxy1402/termuxclaw`
- Git clone: `https://github.com/foxy1402/termuxclaw.git`
- Zip (master): `https://github.com/foxy1402/termuxclaw/archive/refs/heads/master.zip`
- Releases: `https://github.com/foxy1402/termuxclaw/releases`

---

If you want, I can also add:

- A one-command installer (`curl|bash` / `wget|bash`) for Termux
- A simple health-check script for auto-restart if daemon crashes
- A Telegram-first quickstart section for first-time bot users
