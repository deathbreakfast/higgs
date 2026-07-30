//! [`PreflightCheck`] trait for startup validation / seeding steps.

use async_trait::async_trait;
use valence::Valence;

use super::status::{PreflightResult, PreflightStatus};

/// One startup check / seed step.
#[async_trait]
pub trait PreflightCheck: Send + Sync {
    /// Stable id (e.g. `gluon-provider-account`).
    fn name(&self) -> &'static str;

    /// Human-readable description for logs and setup UI.
    fn description(&self) -> &'static str;

    /// Run the check.
    async fn check(&self, valence: &Valence) -> PreflightResult;

    /// Attempt auto-remediation after a non-passed status.
    ///
    /// # Errors
    ///
    /// Returns `anyhow::Error` when remediation itself fails. The default
    /// implementation succeeds with a [`PreflightStatus::Failed`] result stating
    /// auto-remediation is unsupported.
    async fn remediate(&self, valence: &Valence) -> anyhow::Result<PreflightResult> {
        let _ = valence;
        Ok(PreflightResult {
            check_name: self.name().to_string(),
            status: PreflightStatus::Failed {
                message: "auto-remediation not supported".into(),
                details: vec![],
            },
        })
    }

    /// Whether [`Self::remediate`] is implemented for this check.
    fn can_auto_remediate(&self) -> bool {
        false
    }
}
