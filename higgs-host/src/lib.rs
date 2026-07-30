//! # higgs-host — request extraction for server functions
//!
//! Valence router + optional session snapshot for host SSR handlers.
//! Depends on `higgs-identity` only — no product identity schemas. Concrete auth
//! backends and Valence user models are wired via host adapters (e.g.
//! `lepton-host-adapter`).
//!
//! ## Capabilities
//!
//! - **Full request context** — [`HostRequestCtx`] via [`host_ctx`] (feature `ssr`):
//!   router + optional `higgs_identity::SessionSnapshot`
//! - **Data plane only** — [`DataPlaneCtx`] via [`unsafe_data_plane`] (deprecated [`data_plane`])
//! - **Session gate** — [`require_session`] fail-closed for `#[higgs_macros::server(auth)]`
//! - **Operation tagging** — [`with_operation`] / [`current_operation`] for logs and
//!   system-actor attribution (`higgs` / `higgs-macros`)
//!
//! # Organized by task
//!
//! | Task | Start here |
//! |------|------------|
//! | Full request context (router + session) | [`host_ctx`], [`HostRequestCtx`] — [example](#quick-example) |
//! | Data-plane-only context (no session) | [`unsafe_data_plane`], [`DataPlaneCtx`] |
//! | Operation attribution for logs / system actor | [`with_operation`], [`current_operation`] |
//!
//! # Typical request path
//!
//! 1. Host middleware inserts `Extension<SessionSnapshot>` when authenticated.
//! 2. Server function calls [`host_ctx`] (or platform `higgs::Higgs::from_request`).
//! 3. [`HostRequestCtx::actor`] maps session → `User` or missing session → `Anonymous`.
//! 4. Macros / [`with_operation`] set the task-local name used by system Valence.
//!
//! # Feature flags
//!
//! | Feature | What it enables |
//! |---------|-----------------|
//! | `ssr` | [`host_ctx`], [`unsafe_data_plane`], [`HostRequestCtx`], [`DataPlaneCtx`], operation helpers |
//!
//! Without `ssr` this crate exposes no items.
//!
//! # Quick example
//!
//! ```rust,ignore
//! use higgs_host::{host_ctx, HostRequestCtx};
//!
//! async fn handler() -> Result<(), leptos::prelude::ServerFnError> {
//!     let host: HostRequestCtx = host_ctx().await?;
//!     let _actor = host.actor();
//!     Ok(())
//! }
//! ```
//!
//! # Notes
//!
//! - [`host_ctx`] / [`unsafe_data_plane`] return `ServerFnError` when Axum extensions are
//!   missing the Valence `DatabaseRouter` (or extraction otherwise fails).
//! - Session is optional: missing `SessionSnapshot` yields an anonymous actor.

/// SSR request extraction helpers.
#[cfg(feature = "ssr")]
pub mod ssr;

#[cfg(feature = "ssr")]
pub use ssr::{
    current_operation, host_ctx, require_session, unsafe_data_plane, with_operation, DataPlaneCtx,
    HostRequestCtx,
};

#[cfg(feature = "ssr")]
#[allow(deprecated)]
pub use ssr::data_plane;
