//! Process-wide cache of the last preflight run.
//!
//! Prefer consuming the `Vec<PreflightResult>` returned by
//! [`super::PreflightRunner::run_all`] directly. This store exists for hosts that
//! want a later snapshot (e.g. setup UI) without threading results through every layer.

use std::sync::{Mutex, OnceLock};

use super::status::PreflightResult;

static LAST_PREFLIGHT: OnceLock<Mutex<Vec<PreflightResult>>> = OnceLock::new();

fn last_slot() -> &'static Mutex<Vec<PreflightResult>> {
    LAST_PREFLIGHT.get_or_init(|| Mutex::new(Vec::new()))
}

/// Replace stored results (typically after [`super::PreflightRunner::run_all`]).
pub fn store_preflight_results(results: Vec<PreflightResult>) {
    let mut g = last_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *g = results;
}

/// Clone the last stored preflight results (empty if never run).
pub fn preflight_results_snapshot() -> Vec<PreflightResult> {
    last_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}
