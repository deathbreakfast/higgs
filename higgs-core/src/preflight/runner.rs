//! [`PreflightRunner`] — ordered execution of registered checks.

use valence::Valence;

use super::check::PreflightCheck;
use super::status::{PreflightResult, PreflightStatus};
use super::store::store_preflight_results;

/// Runs registered [`PreflightCheck`] implementations in order.
#[derive(Default)]
pub struct PreflightRunner {
    checks: Vec<Box<dyn PreflightCheck>>,
}

impl PreflightRunner {
    /// Create an empty runner with no registered checks.
    pub fn new() -> Self {
        Self { checks: vec![] }
    }

    /// Register a check to run in [`Self::run_all`], in registration order.
    pub fn register(&mut self, check: impl PreflightCheck + 'static) {
        self.checks.push(Box::new(check));
    }

    /// Run all checks, optionally auto-remediating, log, and return results.
    ///
    /// Does not return `Result`: individual check outcomes are [`PreflightStatus`]
    /// values collected into [`PreflightResult`]s. Remediation errors are folded into
    /// [`PreflightStatus::Failed`]. Results are also stored via
    /// [`store_preflight_results`](super::store_preflight_results).
    pub async fn run_all(&self, valence: &Valence) -> Vec<PreflightResult> {
        let mut results = Vec::with_capacity(self.checks.len());
        for check in &self.checks {
            let result = Self::run_one(check.as_ref(), valence).await;
            log_result(check.as_ref(), &result);
            results.push(result);
        }
        store_preflight_results(results.clone());
        results
    }

    async fn run_one(check: &dyn PreflightCheck, valence: &Valence) -> PreflightResult {
        let result = check.check(valence).await;
        if matches!(result.status, PreflightStatus::Passed { .. }) || !check.can_auto_remediate() {
            return result;
        }
        tracing::info!(check = check.name(), "[preflight] Auto-remediating");
        match check.remediate(valence).await {
            Ok(r) => r,
            Err(e) => PreflightResult {
                check_name: check.name().to_string(),
                status: PreflightStatus::Failed {
                    message: format!("remediation failed: {e}"),
                    details: vec![],
                },
            },
        }
    }
}

fn log_result(check: &dyn PreflightCheck, result: &PreflightResult) {
    match &result.status {
        PreflightStatus::Passed { message } => {
            tracing::info!(
                check = check.name(),
                message = %message,
                "[preflight] PASSED"
            );
        }
        PreflightStatus::Warning { message, .. } => {
            tracing::warn!(
                check = check.name(),
                message = %message,
                "[preflight] WARNING"
            );
        }
        PreflightStatus::Failed { message, .. } => {
            tracing::error!(
                check = check.name(),
                message = %message,
                "[preflight] FAILED"
            );
        }
    }
}
