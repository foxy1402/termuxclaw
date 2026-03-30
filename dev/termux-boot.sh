#!/data/data/com.termux/files/usr/bin/sh
#
# ZeroClaw Termux:Boot startup script
#
# Installation:
# 1. Install Termux:Boot from F-Droid or Google Play
# 2. Copy this file to ~/.termux/boot/zeroclaw.sh
# 3. Make it executable: chmod +x ~/.termux/boot/zeroclaw.sh
# 4. Restart your device or run the script manually
#
# Termux:Boot will automatically run this script on Android boot.
#
# Note: This script starts the WATCHDOG which monitors zeroclaw 24/7
# The watchdog automatically restarts zeroclaw if it crashes.
#
# Logs:
# - Watchdog: ~/.zeroclaw/logs/watchdog.log
# - Daemon: ~/.zeroclaw/logs/daemon.log
# - Crashes: ~/.zeroclaw/logs/crashes.log
#
# To view logs: tail -f ~/.zeroclaw/logs/watchdog.log
# To stop: pkill -f "zeroclaw-watchdog"
#

# Ensure the log directory exists
mkdir -p ~/.zeroclaw/logs

# Start the watchdog in the background
# The watchdog will start zeroclaw daemon and keep it running 24/7
exec ~/.zeroclaw/watchdog.sh >> ~/.zeroclaw/logs/boot.log 2>&1 &

