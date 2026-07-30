//! Shared test doubles for workspace crates.

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
