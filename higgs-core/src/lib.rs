//! # Higgs core — Valence / session hub
//!
//! Valence/session hub: config, preflight, and per-request context for server functions.
//!
//! Host wiring for Valence-backed server functions: [`HiggsConfig`], startup [`preflight`],
//! [`server_runtime`], and per-request [`Higgs::from_request`] via `higgs-host` +
//! `higgs-identity`.
//!
//! Provides Valence factory + session-scoped request context. Use the platform `higgs`
//! crate for `ctx.chronon()` / `boson()` / `photon()` accessors.
//!
//! ## Capabilities
//!
//! - **Config** — [`HiggsConfig`] / [`HiggsConfigBuilder`] hold a
//!   [`HiggsValenceFactory`]; build once at host boot and store as
//!   `Arc<HiggsConfig>` in Leptos context
//! - **Factory** — host-implemented [`HiggsValenceFactory`] builds request-scoped
//!   `valence::Valence` from serialized actor JSON
//! - **Request context** — [`Higgs`] (feature `ssr`) via [`Higgs::from_request`];
//!   wraps `higgs_host::HostRequestCtx` plus session user id and config
//! - **Server helpers** — [`server_runtime`] permission-denied payload encode/decode
//!   for `higgs-macros`-generated server functions
//! - **Startup** — [`preflight`] (feature `preflight`) runs validation/seeding once
//!   after the database router is installed
//! - **Actor policy** — [`actor_policy`] helpers such as
//!   [`actor_policy::external_actor_json_policy`] for external actor JSON
//!
//! # Organized by task
//!
//! | Task | Start here |
//! |------|------------|
//! | Boot config / Valence factory | [`HiggsConfig`], [`HiggsConfig::builder`], [`HiggsValenceFactory`] |
//! | Per-request context (SSR) | [`Higgs`], [`Higgs::from_request`] (feature `ssr`) |
//! | Server-function encode/decode helpers | [`server_runtime`] |
//! | Startup validation / seeding | [`preflight`] (feature `preflight`) |
//! | External actor JSON policy | [`actor_policy`] |
//! | Chronon / Boson / Photon accessors | not here — depend on platform crate `higgs` instead |
//!
//! # Typical host flow
//!
//! 1. Implement [`HiggsValenceFactory`].
//! 2. Build [`HiggsConfig`] via [`HiggsConfig::builder`], provide `Arc` in Leptos
//!    context.
//! 3. Call [`Higgs::from_request`] inside server functions (feature `ssr`).
//! 4. Optionally run [`preflight::PreflightRunner`] after the DB router is up.
//!
//! Runnable samples (`higgs` package):
//! `cargo run -p higgs --example config_boot`,
//! `cargo run -p higgs --example shared_factory --features ssr`,
//! `cargo run -p higgs --example preflight_boot --features preflight`.
//! See `higgs/examples/README.md`.
//!
//! For Chronon / Boson / Photon accessors on the same request public crate, depend on
//! platform crate `higgs` instead of this thin hub.
//!
//! # Feature flags
//!
//! | Feature | What it enables |
//! |---------|-----------------|
//! | `ssr` | [`Higgs`], [`Higgs::from_request`], `higgs-host` / Leptos SSR, enables `preflight` |
//! | `preflight` | [`preflight`] module (also enabled by `ssr`) |
//! | `test-utils` | [`test_support`] doubles for downstream tests |
//!
//! # Quick example
//!
//! ```rust,no_run
//! # #[cfg(feature = "test-utils")]
//! # fn boot() -> Result<(), higgs_core::HiggsError> {
//! use std::sync::Arc;
//! use higgs_core::{HiggsConfig, test_support::UnreachableValenceFactory};
//!
//! let config = Arc::new(
//!     HiggsConfig::builder()
//!         .valence_factory(UnreachableValenceFactory)
//!         .build()?,
//! );
//! // provide_context(config) at host boot; Higgs::from_request reads it (feature ssr).
//! # let _ = config;
//! # Ok(())
//! # }
//! ```
//!
//! # Notes
//!
//! - Failures surface as [`HiggsError`] (missing context, internal Valence/actor
//!   errors). See the [crate root example](#quick-example) for builder wiring.

// Clippy sometimes emits `too_long_first_doc_paragraph` without a span on libtest builds.
#![allow(clippy::too_long_first_doc_paragraph)]

/// Actor-JSON policy helpers for shared Valence factories (HIGGS-11).
pub mod actor_policy;
mod config;
mod error;
/// Permission-denied payload encode/decode helpers for server functions.
pub mod server_runtime;
mod valence_factory;

#[cfg(feature = "ssr")]
mod context;
/// Startup validation and idempotent seeding hooks, run once at host boot.
#[cfg(feature = "preflight")]
pub mod preflight;

pub use config::{HiggsConfig, HiggsConfigBuilder};
pub use error::HiggsError;
pub use valence_factory::HiggsValenceFactory;

#[cfg(feature = "ssr")]
pub use context::Higgs;

/// Shared test doubles for downstream crates' own tests.
///
/// For example [`test_support::UnreachableValenceFactory`].
#[cfg(any(test, feature = "test-utils"))]
pub mod test_support;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn valence_factory_is_object_safe() {
        let _: Arc<dyn HiggsValenceFactory> = Arc::new(test_support::UnreachableValenceFactory);
    }

    #[test]
    fn builder_requires_valence_factory() {
        let result = HiggsConfig::builder().build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_succeeds_with_factory() {
        let config = HiggsConfig::builder()
            .valence_factory(test_support::UnreachableValenceFactory)
            .build();
        assert!(config.is_ok());
    }

    #[test]
    fn builder_arc_factory() {
        let factory: Arc<dyn HiggsValenceFactory> = test_support::unreachable_valence_factory();
        let config = HiggsConfig::builder().valence_factory_arc(factory).build();
        assert!(config.is_ok());
    }
}
