#!/data/data/com.termux/files/usr/bin/bash
#
# Emergency fix for SELinux nohup.out API hang issue
#
# This script:
# 1. Stops the watchdog if running
# 2. Removes problematic nohup.out file(s)
# 3. Restarts watchdog with proper output redirection
#
# Usage:
#   chmod +x scripts/fix-nohup-selinux.sh
#   ./scripts/fix-nohup-selinux.sh
#

set -euo pipefail

echo ""
echo "🔧 ZeroClaw SELinux nohup.out Fix"
echo ""
echo "This script fixes Termux:API hangs caused by SELinux blocking"
echo "writes to nohup.out in protected git repo directories."
echo ""

# Stop watchdog if running
if pgrep -f "zeroclaw-watchdog" > /dev/null; then
    echo "⏹️  Stopping existing watchdog..."
    pkill -f "zeroclaw-watchdog" || true
    sleep 2
    echo "✅ Watchdog stopped"
else
    echo "ℹ️  Watchdog not currently running"
fi

# Find and remove problematic nohup.out files
echo ""
echo "🔍 Searching for problematic nohup.out files..."

REMOVED_FILES=0

# Check common locations
for dir in ~/termuxclaw ~/zeroclaw ~/.zeroclaw; do
    if [[ -f "$dir/nohup.out" ]]; then
        echo "  Found: $dir/nohup.out"
        rm -f "$dir/nohup.out"
        echo "  ✅ Removed: $dir/nohup.out"
        ((REMOVED_FILES++))
    fi
done

if [[ $REMOVED_FILES -eq 0 ]]; then
    echo "  ℹ️  No problematic nohup.out files found"
else
    echo "  ✅ Removed $REMOVED_FILES nohup.out file(s)"
fi

# Restart watchdog with safe configuration
echo ""
echo "🚀 Restarting watchdog with safe logging configuration..."

if [[ ! -x ~/.zeroclaw/watchdog.sh ]]; then
    echo "❌ Error: ~/.zeroclaw/watchdog.sh not found or not executable"
    echo "   Run the installer first: ./install.sh"
    exit 1
fi

# Ensure log directory exists
mkdir -p ~/.zeroclaw/logs

# Start watchdog from home directory with explicit output redirection
cd ~
nohup ~/.zeroclaw/watchdog.sh > ~/.zeroclaw/logs/nohup.log 2>&1 &

WATCHDOG_PID=$!

sleep 2

# Verify watchdog started successfully
if kill -0 "$WATCHDOG_PID" 2>/dev/null; then
    echo "✅ Watchdog started successfully (PID: $WATCHDOG_PID)"
    echo ""
    echo "📋 Next steps:"
    echo "  • View logs: tail -f ~/.zeroclaw/logs/watchdog.log"
    echo "  • Test API: zeroclaw chat 'check battery status'"
    echo "  • Stop: pkill -f 'zeroclaw-watchdog'"
    echo ""
    echo "✨ API commands should now work without hanging!"
else
    echo "❌ Error: Watchdog failed to start"
    echo "   Check logs: cat ~/.zeroclaw/logs/nohup.log"
    exit 1
fi
