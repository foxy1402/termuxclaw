use super::traits::{Tool, ToolResult};
use crate::security::SecurityPolicy;
use async_trait::async_trait;
use serde_json::json;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

/// Default maximum Termux command execution time before kill.
const DEFAULT_TERMUX_TIMEOUT_SECS: u64 = 60;
/// Maximum output size in bytes (1MB).
const MAX_OUTPUT_BYTES: usize = 1_048_576;

/// Minimal environment variables needed for Termux binaries.
const SAFE_ENV_VARS: &[&str] = &[
    "PATH", "HOME", "TERM", "LANG", "LC_ALL", "LC_CTYPE", "TMPDIR",
    "PREFIX", "LD_LIBRARY_PATH", "LD_PRELOAD", "ANDROID_DATA", "ANDROID_ROOT", "ANDROID_I18N_ROOT", "ANDROID_TZDATA_ROOT",
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

        if let Some(input) = stdin_payload {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(input.as_bytes()).await.ok();
            }
        }

        let output = match tokio::time::timeout(
            Duration::from_secs(DEFAULT_TERMUX_TIMEOUT_SECS),
            child.wait_with_output(),
        )
        .await
        {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to execute command: {e}")),
                })
            }
            Err(_) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!(
                        "Command timed out after {DEFAULT_TERMUX_TIMEOUT_SECS}s and was killed"
                    )),
                })
            }
        };

        let stdout = Self::truncate_output(String::from_utf8_lossy(&output.stdout).to_string());
        let stderr = Self::truncate_output(String::from_utf8_lossy(&output.stderr).to_string());

        // Detect the well-known Termux:API "Connection refused" error that occurs when
        // commands like `termux-toast` or `termux-notification` are run from a headless
        // background daemon. The API app cannot return results over the local socket when
        // there is no active Termux terminal session. This is non-fatal — report it
        // gracefully so the agent can continue instead of crashing the daemon.
        if stderr.contains("Connection refused") || stderr.contains("ResultReturner") {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(
                    "Termux:API command failed: the API app could not return results because \
                     zeroclaw is running as a background daemon with no active Termux terminal. \
                     Commands like 'toast' and 'notification' require a foreground Termux session. \
                     Try a command that does not require UI feedback (e.g. 'battery-status', \
                     'vibrate', 'sensor-list') or open Termux in the foreground and retry."
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
