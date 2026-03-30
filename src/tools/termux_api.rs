use super::traits::{Tool, ToolResult};
use crate::security::SecurityPolicy;
use async_trait::async_trait;
use serde_json::json;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

/// Default maximum Termux command execution time before kill (reduced from 60s to handle hangs).
const DEFAULT_TERMUX_TIMEOUT_SECS: u64 = 10;
/// Timeout for fast commands that should respond immediately (battery, location, sensor).
const FAST_COMMAND_TIMEOUT_SECS: u64 = 5;
/// Timeout for slow commands (camera, tts, download).
const SLOW_COMMAND_TIMEOUT_SECS: u64 = 30;
/// Maximum retry attempts for hung commands.
const MAX_RETRIES: u32 = 2;
/// Maximum output size in bytes (1MB).
const MAX_OUTPUT_BYTES: usize = 1_048_576;

/// Minimal environment variables needed for Termux binaries.
const SAFE_ENV_VARS: &[&str] = &[
    "PATH",
    "HOME",
    "TERM",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "TMPDIR",
    "PREFIX",
    "LD_LIBRARY_PATH",
    "LD_PRELOAD",
    "ANDROID_DATA",
    "ANDROID_ROOT",
    "ANDROID_I18N_ROOT",
    "ANDROID_TZDATA_ROOT",
];

/// Termux:API command execution tool.
///
/// This tool only executes `termux-*` commands and does not invoke a shell.
pub struct TermuxApiTool {
    security: Arc<SecurityPolicy>,
}

impl TermuxApiTool {
    pub fn new(security: Arc<SecurityPolicy>) -> Self {
        Self { security }
    }

    fn sanitize_command_name(raw: &str) -> Option<String> {
        let mut trimmed = raw.trim();
        if trimmed.starts_with("termux-") {
            trimmed = trimmed.strip_prefix("termux-").unwrap();
        }
        if trimmed.is_empty() {
            return None;
        }
        if trimmed.starts_with('-') || trimmed.contains('/') || trimmed.contains('\\') {
            return None;
        }
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return None;
        }
        Some(trimmed.to_string())
    }

    fn truncate_output(mut s: String) -> String {
        if s.len() <= MAX_OUTPUT_BYTES {
            return s;
        }
        let mut b = MAX_OUTPUT_BYTES.min(s.len());
        while b > 0 && !s.is_char_boundary(b) {
            b -= 1;
        }
        s.truncate(b);
        s.push_str("\n... [output truncated at 1MB]");
        s
    }

    /// Determine appropriate timeout based on command type.
    fn get_command_timeout(command: &str) -> Duration {
        // Fast commands that should respond immediately
        let fast_commands = &[
            "battery-status",
            "brightness",
            "clipboard-get",
            "clipboard-set",
            "contact-list",
            "dialog",
            "infrared-frequencies",
            "infrared-transmit",
            "location",
            "notification",
            "notification-remove",
            "sensor-list",
            "telephony-cellinfo",
            "telephony-deviceinfo",
            "toast",
            "torch",
            "vibrate",
            "volume",
            "wifi-connectioninfo",
            "wifi-scaninfo",
        ];

        // Slow commands that may take longer
        let slow_commands = &[
            "camera-photo",
            "camera-info",
            "download",
            "fingerprint",
            "media-player",
            "media-scan",
            "microphone-record",
            "share",
            "sms-send",
            "storage-get",
            "telephony-call",
            "tts-speak",
            "usb",
        ];

        if fast_commands.contains(&command) {
            Duration::from_secs(FAST_COMMAND_TIMEOUT_SECS)
        } else if slow_commands.contains(&command) {
            Duration::from_secs(SLOW_COMMAND_TIMEOUT_SECS)
        } else {
            Duration::from_secs(DEFAULT_TERMUX_TIMEOUT_SECS)
        }
    }

    /// Attempt to restart Termux:API service using Android's activity manager.
    async fn restart_termux_api_service() -> bool {
        // First, force-stop the Termux:API app
        let stop_result = tokio::process::Command::new("am")
            .args(["force-stop", "com.termux.api"])
            .output()
            .await;

        if stop_result.is_err() {
            return false;
        }

        // Wait for service to fully stop
        tokio::time::sleep(Duration::from_millis(1000)).await;

        // Start the service by calling a simple command (this triggers service startup)
        let start_result = tokio::process::Command::new("termux-vibrate")
            .args(["-d", "1"]) // Vibrate for 1ms (minimal disturbance)
            .output()
            .await;

        start_result.is_ok()
    }

    /// Execute with retry logic to handle Termux API hangs.
    async fn execute_with_retry(
        &self,
        command: &str,
        args: &serde_json::Value,
    ) -> anyhow::Result<ToolResult> {
        let mut last_error = None;
        let mut service_restarted = false;

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                // On second retry (attempt == 2), try restarting Termux:API service
                if attempt == 2 && !service_restarted {
                    service_restarted = true;
                    if Self::restart_termux_api_service().await {
                        // Service restarted successfully, wait a bit longer
                        tokio::time::sleep(Duration::from_millis(1500)).await;
                    } else {
                        // Couldn't restart, use normal backoff
                        let backoff_ms = 500 * (1 << (attempt - 1));
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    }
                } else {
                    // Exponential backoff: 500ms, 1s
                    let backoff_ms = 500 * (1 << (attempt - 1));
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                }
            }

            match self.execute_single_attempt(command, args).await {
                Ok(result) if result.success => return Ok(result),
                Ok(result)
                    if result
                        .error
                        .as_ref()
                        .map_or(false, |e| e.contains("timed out")) =>
                {
                    last_error = Some(result.error.unwrap_or_default());
                }
                Ok(result) => return Ok(result), // Non-timeout error, don't retry
                Err(e) => {
                    last_error = Some(e.to_string());
                }
            }
        }

        let mut error_msg = format!(
            "Command failed after {} retries. Last error: {}",
            MAX_RETRIES,
            last_error.unwrap_or_else(|| "unknown error".to_string())
        );

        if service_restarted {
            error_msg.push_str(
                "\nTermux:API service was automatically restarted during retry attempts.",
            );
        }

        Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some(error_msg),
        })
    }

    /// Execute a single command attempt with timeout and process cleanup.
    async fn execute_single_attempt(
        &self,
        command: &str,
        args: &serde_json::Value,
    ) -> anyhow::Result<ToolResult> {
        let exec = format!("termux-{}", command);
        let mut cmd = tokio::process::Command::new(&exec);

        if let Some(arr) = args.get("args").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    cmd.arg(s);
                }
            }
        }

        let stdin_payload = args
            .get("stdin")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        if stdin_payload.is_some() {
            cmd.stdin(Stdio::piped());
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(&self.security.workspace_dir);
        cmd.env_clear();
        for var in SAFE_ENV_VARS {
            if let Ok(val) = std::env::var(var) {
                cmd.env(var, val);
            }
        }

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                let hint = if e.kind() == std::io::ErrorKind::NotFound {
                    "Termux:API command not found. Install Termux:API app and run `pkg install termux-api`."
                } else {
                    "Failed to spawn termux command."
                };
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("{hint} ({e})")),
                });
            }
        };

        // Write stdin if provided
        if let Some(input) = stdin_payload {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(input.as_bytes()).await;
            }
        }

        // Use command-specific timeout
        let timeout = Self::get_command_timeout(command);

        let output = match tokio::time::timeout(timeout, child.wait_with_output()).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to execute command: {e}")),
                })
            }
            Err(_) => {
                // Timeout occurred. wait_with_output consumes the child process handle,
                // so we can't reliably send a kill signal here.
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!(
                        "Command '{}' timed out after {:?}. \
                         Termux:API may be unresponsive. Try restarting Termux:API app or device.",
                        exec, timeout
                    )),
                });
            }
        };

        let stdout = Self::truncate_output(String::from_utf8_lossy(&output.stdout).to_string());
        let stderr = Self::truncate_output(String::from_utf8_lossy(&output.stderr).to_string());

        // Detect Termux:API connection issues
        if stderr.contains("Connection refused") || stderr.contains("ResultReturner") {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(
                    "Termux:API connection refused: the API app could not return results. \
                     This usually happens when zeroclaw runs as a background daemon. \
                     Try: (1) Open Termux in foreground, (2) Restart Termux:API app, \
                     (3) Use commands that don't require UI (battery-status, vibrate, sensor-list)."
                        .into(),
                ),
            });
        }

        Ok(ToolResult {
            success: output.status.success(),
            output: stdout,
            error: if stderr.is_empty() {
                None
            } else {
                Some(stderr)
            },
        })
    }
}

#[async_trait]
impl Tool for TermuxApiTool {
    fn name(&self) -> &str {
        "termux_api"
    }

    fn description(&self) -> &str {
        "Execute Termux:API commands (termux-*) to interact with Android device features"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Termux API command without the 'termux-' prefix (e.g. 'battery-status', 'vibrate', 'toast', 'camera-photo')"
                },
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Arguments to pass to the termux command"
                },
                "stdin": {
                    "type": "string",
                    "description": "Optional stdin content to pipe into the command"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter"))?;
        let command = match Self::sanitize_command_name(command) {
            Some(cmd) => cmd,
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(
                        "Invalid command name. Use the command without 'termux-' and only lowercase letters, digits, and dashes."
                            .into(),
                    ),
                })
            }
        };

        if !self.security.can_act() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("blocked by security policy: autonomy is read-only".into()),
            });
        }

        if self.security.is_rate_limited() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Rate limit exceeded: too many actions in the last hour".into()),
            });
        }

        if !self.security.record_action() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Rate limit exceeded: action budget exhausted".into()),
            });
        }

        // Execute with retry logic to handle Termux API hangs
        self.execute_with_retry(&command, &args).await
    }
}
