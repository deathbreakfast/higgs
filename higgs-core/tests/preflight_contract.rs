//! Integration tests for startup preflight runner contracts.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use higgs_core::preflight::{
    preflight_results_snapshot, store_preflight_results, PreflightCheck, PreflightResult,
    PreflightRunner, PreflightStatus,
};
use valence::{
    DatabaseRouter, InMemoryBackend, RouterValenceFactory, RouterValenceFactoryConfig, Valence,
    DEFAULT_IN_MEMORY_ROUTER_KEY,
};

/// Process-global preflight store is shared; serialize sync snapshot tests.
static PREFLIGHT_STORE_LOCK: Mutex<()> = Mutex::new(());

fn mem_valence() -> Valence {
    let mut router = DatabaseRouter::new();
    router.register(
        DEFAULT_IN_MEMORY_ROUTER_KEY.to_string(),
        Arc::new(InMemoryBackend::new()),
    );
    let factory = RouterValenceFactory::arc(
        Arc::new(router),
        RouterValenceFactoryConfig::new(DEFAULT_IN_MEMORY_ROUTER_KEY),
    );
    match factory.build(&serde_json::json!({"Anonymous": null})) {
        Ok(v) => v,
        Err(e) => panic!("mem valence: {e}"),
    }
}

struct PassCheck;

#[async_trait]
impl PreflightCheck for PassCheck {
    fn name(&self) -> &'static str {
        "pass-check"
    }

    fn description(&self) -> &'static str {
        "always passes"
    }

    async fn check(&self, _valence: &Valence) -> PreflightResult {
        PreflightResult {
            check_name: self.name().to_string(),
            status: PreflightStatus::Passed {
                message: "ok".into(),
            },
        }
    }
}

struct FailCheck;

#[async_trait]
impl PreflightCheck for FailCheck {
    fn name(&self) -> &'static str {
        "fail-check"
    }

    fn description(&self) -> &'static str {
        "always fails"
    }

    async fn check(&self, _valence: &Valence) -> PreflightResult {
        PreflightResult {
            check_name: self.name().to_string(),
            status: PreflightStatus::Failed {
                message: "broken".into(),
                details: vec!["detail".into()],
            },
        }
    }
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // process-global preflight store; serialize with sync snapshot test
async fn preflight_runner_empty_happy_path() {
    let _guard = PREFLIGHT_STORE_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let runner = PreflightRunner::new();
    let valence = mem_valence();
    let results = runner.run_all(&valence).await;
    assert!(results.is_empty());
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // process-global preflight store; serialize with sync snapshot test
async fn preflight_runner_pass_happy_path() {
    let _guard = PREFLIGHT_STORE_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut runner = PreflightRunner::new();
    runner.register(PassCheck);
    let valence = mem_valence();
    let results = runner.run_all(&valence).await;
    assert_eq!(results.len(), 1);
    assert!(matches!(
        results[0].status,
        PreflightStatus::Passed { ref message } if message == "ok"
    ));
    assert_eq!(results[0].check_name, "pass-check");
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // process-global preflight store; serialize with sync snapshot test
async fn preflight_runner_failed_check_sad() {
    let _guard = PREFLIGHT_STORE_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut runner = PreflightRunner::new();
    runner.register(FailCheck);
    let valence = mem_valence();
    let results = runner.run_all(&valence).await;
    assert_eq!(results.len(), 1);
    match &results[0].status {
        PreflightStatus::Failed { message, details } => {
            assert_eq!(message, "broken");
            assert_eq!(details.as_slice(), ["detail"]);
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

#[test]
fn preflight_store_snapshot_happy_path() {
    let _guard = PREFLIGHT_STORE_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    store_preflight_results(vec![PreflightResult {
        check_name: "stored".into(),
        status: PreflightStatus::Passed {
            message: "cached".into(),
        },
    }]);
    let snap = preflight_results_snapshot();
    assert_eq!(snap.len(), 1);
    assert_eq!(snap[0].check_name, "stored");
}
