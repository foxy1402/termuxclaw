#!/data/data/com.termux/files/usr/bin/bash
#
# ZeroClaw Watchdog - Keeps zeroclaw running 24/7
#
# This script monitors the zeroclaw daemon process and automatically restarts it if:
# - Process crashes (exit code != 0)
# - Process exits unexpectedly
# - Process becomes unresponsive (optional healthcheck)
#
# Features:
# - Automatic restart with exponential backoff
# - Crash logging with timestamps
# - Process health monitoring
# - Graceful shutdown handling
# - Survives Termux app restarts
#
# Installation:
# 1. Copy to ~/.zeroclaw/watchdog.sh
# 2. Make executable: chmod +x ~/.zeroclaw/watchdog.sh
# 3. Run in background: nohup ~/.zeroclaw/watchdog.sh &
# 4. Or use Termux:Boot (see termux-boot.sh)
#

set -euo pipefail

# ──────────────────────────────────────────────────────────────
# Configuration
# ──────────────────────────────────────────────────────────────

ZEROCLAW_BIN="${ZEROCLAW_BIN:-zeroclaw}"
ZEROCLAW_CMD="${ZEROCLAW_CMD:-daemon}"
ZEROCLAW_ARGS="${ZEROCLAW_ARGS:-}"

WORKSPACE_DIR="${HOME}/.zeroclaw"
LOG_DIR="${WORKSPACE_DIR}/logs"
WATCHDOG_LOG="${LOG_DIR}/watchdog.log"
DAEMON_LOG="${LOG_DIR}/daemon.log"
CRASH_LOG="${LOG_DIR}/crashes.log"
PID_FILE="${WORKSPACE_DIR}/watchdog.pid"
HEALTHCHECK_INTERVAL="${HEALTHCHECK_INTERVAL:-60}"  # seconds
MAX_RESTARTS_PER_HOUR="${MAX_RESTARTS_PER_HOUR:-10}"

# Backoff configuration
INITIAL_BACKOFF=5      # seconds
MAX_BACKOFF=300        # 5 minutes
CURRENT_BACKOFF=$INITIAL_BACKOFF

# ──────────────────────────────────────────────────────────────
# Utilities
# ──────────────────────────────────────────────────────────────

log() {
    local level="$1"
    shift
    local msg="$*"
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[${timestamp}] [${level}] ${msg}" | tee -a "$WATCHDOG_LOG"
}

log_crash() {
    local exit_code="$1"
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[${timestamp}] ZeroClaw crashed with exit code: ${exit_code}" >> "$CRASH_LOG"
}

ensure_dirs() {
    mkdir -p "$LOG_DIR"
}

check_already_running() {
    if [[ -f "$PID_FILE" ]]; then
        local old_pid
        old_pid=$(cat "$PID_FILE")
        if kill -0 "$old_pid" 2>/dev/null; then
            log "ERROR" "Watchdog already running with PID $old_pid"
            exit 1
        else
            log "WARN" "Stale PID file found, removing"
            rm -f "$PID_FILE"
        fi
    fi
}

write_pid_file() {
    echo $$ > "$PID_FILE"
}

cleanup_pid_file() {
    rm -f "$PID_FILE"
}

# ──────────────────────────────────────────────────────────────
# Health Check
# ──────────────────────────────────────────────────────────────

check_daemon_health() {
    # Check if daemon_state.json is being updated
    local state_file="${WORKSPACE_DIR}/daemon_state.json"
    
    if [[ ! -f "$state_file" ]]; then
        log "WARN" "Health check: daemon_state.json missing"
        return 1
    fi
    
    # Check if state file was updated in last 30 seconds
    local now
    now=$(date +%s)
    local file_mtime
    file_mtime=$(stat -c %Y "$state_file" 2>/dev/null || stat -f %m "$state_file" 2>/dev/null || echo 0)
    local age=$((now - file_mtime))
    
    if [[ $age -gt 30 ]]; then
        log "WARN" "Health check: daemon_state.json not updated in ${age}s (stale)"
        return 1
    fi
    
    # Check if health status shows errors
    if command -v jq >/dev/null 2>&1; then
        local health_status
        health_status=$(jq -r '.daemon.status // "unknown"' "$state_file" 2>/dev/null || echo "unknown")
        if [[ "$health_status" == "error" ]]; then
            log "WARN" "Health check: daemon status is 'error'"
            return 1
        fi
    fi
    
    return 0
}

# ──────────────────────────────────────────────────────────────
# Restart Rate Limiting
# ──────────────────────────────────────────────────────────────

RESTART_TIMES_FILE="${WORKSPACE_DIR}/.restart_times"

record_restart() {
    local now
    now=$(date +%s)
    echo "$now" >> "$RESTART_TIMES_FILE"
    
    # Clean up restart times older than 1 hour
    local one_hour_ago=$((now - 3600))
    if [[ -f "$RESTART_TIMES_FILE" ]]; then
        local temp_file
        temp_file=$(mktemp)
        while IFS= read -r timestamp; do
            if [[ $timestamp -gt $one_hour_ago ]]; then
                echo "$timestamp" >> "$temp_file"
            fi
        done < "$RESTART_TIMES_FILE"
        mv "$temp_file" "$RESTART_TIMES_FILE"
    fi
}

check_restart_rate_limit() {
    if [[ ! -f "$RESTART_TIMES_FILE" ]]; then
        return 0
    fi
    
    local now
    now=$(date +%s)
    local one_hour_ago=$((now - 3600))
    local restart_count=0
    
    while IFS= read -r timestamp; do
        if [[ $timestamp -gt $one_hour_ago ]]; then
            ((restart_count++))
        fi
    done < "$RESTART_TIMES_FILE"
    
    if [[ $restart_count -ge $MAX_RESTARTS_PER_HOUR ]]; then
        log "ERROR" "Restart rate limit exceeded: $restart_count restarts in last hour (max: $MAX_RESTARTS_PER_HOUR)"
        log "ERROR" "Entering extended backoff: 1 hour"
        sleep 3600
        # Clear restart history after extended backoff
        rm -f "$RESTART_TIMES_FILE"
        return 1
    fi
    
    return 0
}

# ──────────────────────────────────────────────────────────────
# Signal Handlers
# ──────────────────────────────────────────────────────────────

SHUTTING_DOWN=0

handle_shutdown() {
    SHUTTING_DOWN=1
    log "INFO" "Watchdog shutting down gracefully..."
    
    # Kill zeroclaw daemon if running
    if [[ -n "${ZEROCLAW_PID:-}" ]]; then
        log "INFO" "Stopping zeroclaw daemon (PID: $ZEROCLAW_PID)..."
        kill -TERM "$ZEROCLAW_PID" 2>/dev/null || true
        wait "$ZEROCLAW_PID" 2>/dev/null || true
    fi
    
    cleanup_pid_file
    log "INFO" "Watchdog stopped"
    exit 0
}

trap handle_shutdown SIGINT SIGTERM

# Ignore SIGHUP so watchdog survives terminal disconnects
trap '' SIGHUP

# ──────────────────────────────────────────────────────────────
# Main Watchdog Loop
# ──────────────────────────────────────────────────────────────

main() {
    ensure_dirs
    check_already_running
    write_pid_file
    
    log "INFO" "ZeroClaw Watchdog starting..."
    log "INFO" "Command: $ZEROCLAW_BIN $ZEROCLAW_CMD $ZEROCLAW_ARGS"
    log "INFO" "Logs: $DAEMON_LOG"
    log "INFO" "Healthcheck interval: ${HEALTHCHECK_INTERVAL}s"
    log "INFO" "Max restarts per hour: $MAX_RESTARTS_PER_HOUR"
    
    local consecutive_failures=0
    
    while [[ $SHUTTING_DOWN -eq 0 ]]; do
        log "INFO" "Starting zeroclaw daemon..."
        
        # Start zeroclaw daemon in background
        $ZEROCLAW_BIN $ZEROCLAW_CMD $ZEROCLAW_ARGS >> "$DAEMON_LOG" 2>&1 &
        ZEROCLAW_PID=$!
        
        log "INFO" "ZeroClaw daemon started (PID: $ZEROCLAW_PID)"
        
        # Reset backoff on successful start
        CURRENT_BACKOFF=$INITIAL_BACKOFF
        consecutive_failures=0
        
        # Monitor the process
        local last_health_check=0
        
        while kill -0 "$ZEROCLAW_PID" 2>/dev/null; do
            sleep 5
            
            # Periodic health check
            local now
            now=$(date +%s)
            if [[ $((now - last_health_check)) -ge $HEALTHCHECK_INTERVAL ]]; then
                if ! check_daemon_health; then
                    log "ERROR" "Health check failed, restarting daemon..."
                    kill -TERM "$ZEROCLAW_PID" 2>/dev/null || true
                    break
                fi
                last_health_check=$now
            fi
        done
        
        # Process exited, check exit code
        wait "$ZEROCLAW_PID" 2>/dev/null
        local exit_code=$?
        
        if [[ $SHUTTING_DOWN -eq 1 ]]; then
            break
        fi
        
        # Log crash
        log_crash "$exit_code"
        
        if [[ $exit_code -eq 0 ]]; then
            log "WARN" "ZeroClaw exited cleanly (exit code 0)"
        else
            log "ERROR" "ZeroClaw crashed (exit code: $exit_code)"
            ((consecutive_failures++))
        fi
        
        # Check restart rate limit
        if ! check_restart_rate_limit; then
            continue
        fi
        
        record_restart
        
        # Calculate backoff
        if [[ $consecutive_failures -gt 0 ]]; then
            log "WARN" "Restarting after ${CURRENT_BACKOFF}s backoff (consecutive failures: $consecutive_failures)..."
            sleep "$CURRENT_BACKOFF"
            
            # Exponential backoff: double each failure, cap at MAX_BACKOFF
            CURRENT_BACKOFF=$((CURRENT_BACKOFF * 2))
            if [[ $CURRENT_BACKOFF -gt $MAX_BACKOFF ]]; then
                CURRENT_BACKOFF=$MAX_BACKOFF
            fi
        else
            log "INFO" "Restarting immediately..."
        fi
    done
    
    cleanup_pid_file
}

# ──────────────────────────────────────────────────────────────
# Entry Point
# ──────────────────────────────────────────────────────────────

main "$@"
