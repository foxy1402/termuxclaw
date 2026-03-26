# Termux Bootstrap (Android)

This fork is maintained for **Termux-first** usage on Android.

Last verified: **March 26, 2026**.

## Recommended install path

```bash
pkg update -y
pkg install -y git rust termux-api
termux-setup-storage

git clone https://github.com/zeroclaw-labs/zeroclaw.git
cd zeroclaw
./install.sh
```

`install.sh` builds and installs `zeroclaw`, then starts onboarding unless skipped.

## Non-interactive onboarding

```bash
./install.sh --api-key "sk-..." --provider openrouter
```

Or with env vars:

```bash
ZEROCLAW_API_KEY="sk-..." ZEROCLAW_PROVIDER="openrouter" ./install.sh
```

## Build resource guidance (Termux)

- Minimum: **2 GB RAM + swap**, **6 GB free disk**
- Recommended: **4 GB+ RAM + swap**, **10 GB free disk**

If local resources are tight, use:

```bash
./install.sh --prefer-prebuilt
```

To require binary-only install:

```bash
./install.sh --prebuilt-only
```

## Auto-start on boot (Termux:Boot)

```bash
mkdir -p ~/.termux/boot
cat > ~/.termux/boot/zeroclaw.sh <<'SH'
#!/data/data/com.termux/files/usr/bin/sh
termux-wake-lock
zeroclaw daemon
SH
chmod +x ~/.termux/boot/zeroclaw.sh
```

Also disable battery optimization for both Termux and Termux:Boot in Android settings.

## Validate installation

```bash
zeroclaw --version
zeroclaw status
zeroclaw doctor
```