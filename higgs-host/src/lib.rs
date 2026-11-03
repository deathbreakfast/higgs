//! # higgs-host — request extraction for server functions
//!
//! Valence router + optional session snapshot for host SSR handlers.
//! Concrete auth backends and Valence user models are wired via host adapters.
//!
//! ## Features
//!
//! - **Full request context** — `host_ctx` (feature `ssr`) loads the Valence router and
//!   optional session snapshot into `HostRequestCtx` for server functions and extractors.
//!   [Quick example](#quick-example)
//! - **Data plane only** — `unsafe_data_plane` returns the router without a session actor
//!   for boot or control-plane paths. Prefer session-scoped Valence for user CRUD.
//!   [Data-plane only](#data-plane-only)
//! - **Session gate** — `require_session` fails closed when `#[higgs_macros::server(auth)]`
//!   needs a signed-in `SessionSnapshot`. [Quick example](#quick-example)
//! - **Operation tagging** — `with_operation` / `current_operation` set a task-local name
//!   for logs and `unsafe_system_valence` attribution.
//!   [Operation attribution](#operation-attribution)
//!
//! # Getting started
//!
//! Host middleware inserts the session snapshot; server functions call `host_ctx` (or
//! platform `higgs::Higgs::from_request`) to map that snapshot to a Valence actor.
//!
//! 1. Host middleware inserts `Extension<SessionSnapshot>` when authenticated.
//! 2. Server function calls `host_ctx` (or platform `higgs::Higgs::from_request`).
//! 3. `HostRequestCtx::actor` maps session → `User` or missing session → `Anonymous`.
//! 4. Macros / `with_operation` set the task-local name used by system Valence.
//!
//! Observable first success: `cargo run -p higgs --example axum_session_host --features ssr`
//! prints `axum_session_host: OK — session → Higgs → worker factory`.
//!
//! # Quick example
//!
//! Provides full host context in a server function per request: router plus optional
//! session mapped to a Valence actor (`User` or `Anonymous`).
//!
//! Prerequisites: feature `ssr`, Valence `DatabaseRouter` in Axum extensions.
//!
//! ```rust,ignore
//! use higgs_host::{host_ctx, HostRequestCtx};
//! use valence::Actor;
//!
//! async fn handler() -> Result<(), leptos::prelude::ServerFnError> {
//!     let host: HostRequestCtx = host_ctx().await?;
//!     let actor = host.actor();
//!     // Missing SessionSnapshot → Anonymous; present snapshot → User.
//!     assert!(matches!(
//!         actor,
//!         Actor::Anonymous | Actor::User { .. } | Actor::System { .. }
//!     ));
//!     Ok(())
//! }
//! ```
//!
//! Observable outcome: `host.actor()` is `User` when a snapshot is present, otherwise
//! `Anonymous`. End-to-end: `cargo run -p higgs --example axum_session_host --features ssr`.
//! Errors: missing router extension → `ServerFnError`. Next: [Data-plane only](#data-plane-only)
//! or [Operation attribution](#operation-attribution).
//!
//! # Feature flags
//!
//! | Feature | What it enables |
//! |---------|-----------------|
//! | `ssr` | `host_ctx`, `unsafe_data_plane`, `HostRequestCtx`, `DataPlaneCtx`, operation helpers |
//!
//! Without `ssr` this crate exposes no items.
//!
//! # Data-plane only
//!
//! Gives the Valence router alone when boot or control-plane code must not carry a session
//! actor. Prefer `higgs::Higgs::valence` for user-scoped CRUD.
//!
//! Prerequisites: feature `ssr`. Call `unsafe_data_plane` when you need the Valence
//! router without a session actor (boot/control-plane). Prefer session-scoped Valence
//! via `higgs::Higgs::valence` for user CRUD.
//!
//! ```rust,ignore
//! use higgs_host::unsafe_data_plane;
//!
//! let plane = unsafe_data_plane().await?;
//! assert!(std::sync::Arc::strong_count(&plane.database_router) >= 1);
//! ```
//!
//! Errors: missing `DatabaseRouter` extension → `ServerFnError`. Deprecated alias:
//! `data_plane`. Next: [Quick example](#quick-example) for full context, or
//! [Operation attribution](#operation-attribution).
//!
//! # Operation attribution
//!
//! Sets a stable operation name on async work so logs and `unsafe_system_valence` see the
//! same string for the duration of the future.
//!
//! Prerequisites: feature `ssr`. Wrap work in `with_operation` so `current_operation`
//! (and `higgs::Higgs::unsafe_system_valence`) see a stable name.
//!
//! ```rust
//! # async fn demo() {
//! use higgs_host::{current_operation, with_operation};
//!
//! let seen = with_operation("ops.create", async { current_operation() }).await;
//! assert_eq!(seen, Some("ops.create"));
//! # }
//! ```
//!
//! Observable outcome: nested `with_operation` returns `Some("ops.create")` from
//! `current_operation`. Next: [Quick example](#quick-example).
//!
//! # Notes
//!
//! - `host_ctx` / `unsafe_data_plane` return `ServerFnError` when Axum extensions are
//!   missing the Valence `DatabaseRouter` (or extraction otherwise fails).
//! - Session is optional: missing `SessionSnapshot` yields an anonymous actor.
//! - Depends on `higgs-identity` for the session snapshot contract only.

/// SSR request extraction helpers.
#[cfg(feature = "ssr")]
pub mod ssr;

#[cfg(feature = "ssr")]
pub use ssr::{
    current_operation, host_ctx, require_session, unsafe_data_plane, with_operation, DataPlaneCtx,
    HostRequestCtx, SystemActorRejected,
};

#[cfg(feature = "ssr")]
#[allow(deprecated)]
pub use ssr::data_plane;
