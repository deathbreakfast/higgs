//! Test doubles for configuring [`crate::HiggsConfig`] without a live Valence backend.
//!
//! Enable with Cargo feature `test-utils` (or use these types from in-crate `#[cfg(test)]`).
//! Downstream crates depend on `higgs-core` / `higgs` with `test-utils` and install
//! `UnreachableValenceFactory` when the test only needs a built config, not
//! [`HiggsValenceFactory::build`].
//!
//! ```rust,no_run
//! # #[cfg(feature = "test-utils")]
//! # fn demo() -> Result<(), higgs_core::HiggsError> {
//! use higgs_core::{HiggsConfig, test_support::UnreachableValenceFactory};
//!
//! let config = HiggsConfig::builder()
//!     .valence_factory(UnreachableValenceFactory)
//!     .build()?;
//! assert!(std::sync::Arc::strong_count(config.valence_factory()) >= 1);
//! # Ok(())
//! # }
//! ```
//!
//! Calling `UnreachableValenceFactory::build` panics — intentional so accidental Valence
//! construction fails loudly in unit tests.

use std::sync::Arc;

use crate::HiggsValenceFactory;

/// Factory that panics if [`HiggsValenceFactory::build`] is called.
///
/// Use when a test needs a configured [`crate::HiggsConfig`] but never builds Valence.
#[derive(Debug)]
pub struct UnreachableValenceFactory;

impl HiggsValenceFactory for UnreachableValenceFactory {
    fn build(&self, _actor_json: &serde_json::Value) -> anyhow::Result<valence::Valence> {
        unreachable!("UnreachableValenceFactory::build — not used in this test harness")
    }
}

/// Convenience `Arc` wrapper around [`UnreachableValenceFactory`].
pub fn unreachable_valence_factory() -> Arc<dyn HiggsValenceFactory> {
    Arc::new(UnreachableValenceFactory)
}
