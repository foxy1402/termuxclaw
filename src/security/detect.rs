//! Auto-detection of available security features

use crate::config::SecurityConfig;
use crate::security::traits::Sandbox;
use std::sync::Arc;

/// Create a sandbox based on auto-detection or explicit config
///
/// On Termux/Android, sandboxing is handled by Android's app sandbox.
/// No additional OS-level sandboxing backends are supported.
pub fn create_sandbox(_config: &SecurityConfig) -> Arc<dyn Sandbox> {
    // On Termux, always use NoopSandbox - Android app sandbox provides isolation
    tracing::info!("Using Android app sandbox (no additional OS-level sandboxing on Termux)");
    Arc::new(super::traits::NoopSandbox)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SecurityConfig;

    #[test]
    fn create_sandbox_returns_noop() {
        let config = SecurityConfig::default();
        let sandbox = create_sandbox(&config);
        // Termux always uses NoopSandbox (Android app sandbox provides isolation)
        assert_eq!(sandbox.name(), "none");
        assert!(sandbox.is_available());
    }

    #[test]
    fn sandbox_is_always_available() {
        let config = SecurityConfig::default();
        let sandbox = create_sandbox(&config);
        // Should always return a usable sandbox on Termux
        assert!(sandbox.is_available());
    }
}
