//! Startup validation and idempotent seeding hooks (preflight).
//!
//! Run a [`PreflightRunner`](crate::preflight::PreflightRunner) once during host startup
//! after the global database router is installed and before background schedulers, so
//! seeding work is visible and structured instead of only via Chronon `RunOnce` jobs.
//!
//! Runnable: `cargo run -p higgs --example preflight_boot --features preflight`
//!
//! # Examples
//!
//! ```rust,ignore
//! use higgs_core::preflight::{PreflightCheck, PreflightRunner};
//!
//! let mut runner = PreflightRunner::new();
//! runner.register(MyCheck);
//! let results = runner.run_all(&valence).await;
//! let _ = results;
//! ```

mod check;
mod runner;
mod status;
mod store;

pub use check::PreflightCheck;
pub use runner::PreflightRunner;
pub use status::{PreflightResult, PreflightStatus};
pub use store::{preflight_results_snapshot, store_preflight_results};
