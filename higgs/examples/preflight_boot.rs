//! Startup preflight: register a check and run `PreflightRunner::run_all`.
//!
//! Standalone — hosts retain the returned results (e.g. in `AppState`) and auth-gate
//! any setup UI that exposes them.
//!
//! ```bash
//! cargo run -p higgs --example preflight_boot --features preflight
//! ```

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use async_trait::async_trait;
use higgs::preflight::{PreflightCheck, PreflightResult, PreflightRunner, PreflightStatus};
use valence::{InMemoryBackend, Valence};

struct AlwaysPass;

#[async_trait]
impl PreflightCheck for AlwaysPass {
    fn name(&self) -> &'static str {
        "demo-always-pass"
    }

    fn description(&self) -> &'static str {
        "example check that always passes"
    }

    async fn check(&self, _valence: &Valence) -> PreflightResult {
        PreflightResult {
            check_name: self.name().to_string(),
            status: PreflightStatus::Passed {
                message: "demo ok".into(),
            },
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let valence = Valence::builder()
        .add_backend("default", Arc::new(InMemoryBackend::new()))
        .build()?;

    let mut runner = PreflightRunner::new();
    runner.register(AlwaysPass);
    let results = runner.run_all(&valence).await;

    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].status, PreflightStatus::Passed { .. }));

    println!(
        "preflight: {} — {:?}",
        results[0].check_name, results[0].status
    );
    Ok(())
}
