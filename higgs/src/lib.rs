//! # Higgs — platform host wiring
//!
//! Process-wide [`HiggsConfig`] and per-request [`Higgs`] for hosts:
//! one Valence factory shared by server functions, Chronon jobs, Boson tasks, and
//! Photon handlers, plus optional Chronon / Boson / Photon accessors when those
//! backends are registered at boot.
//!
//! [`higgs_core`] provides Valence/session config and request context. This crate
//! adds Chronon / Boson / Photon accessors so `ctx.chronon()` / `boson()` / `photon()`
//! work after backends are registered at boot. Depend on this crate as package name
//! **`higgs`** so `higgs-macros` keeps resolving `higgs::Higgs::from_request()`.
//!
//! # Security notes
//!
//! - Prefer [`Higgs::valence`] for user-driven data access after
//!   [`require_session`] / `#[higgs_macros::server(auth)]`.
//! - [`Higgs::unsafe_system_valence`] mints `Actor::System` and **bypasses Valence
//!   privacy**. Do not use it to paper over missing policies or auth gates. There is
//!   no soft-named `system_valence` alias.
//! - Raw router access is [`Higgs::unsafe_database_router`] / [`higgs_host::unsafe_data_plane`].
//! - Install [`actor_policy::external_actor_json_policy`] on factories that rebuild
//!   Valence from untrusted actor JSON (see repository `SECURITY.md`).
//! - Use `#[higgs_macros::server(auth)]` for endpoints that require a signed-in session.
//!
//! ## Capabilities
//!
//! - **Shared Valence factory** — implement [`HiggsValenceFactory`], store it on
//!   [`HiggsConfig`]; also expose `valence::ValenceFactory` so Chronon / Boson /
//!   Photon workers rebuild Valence from the same factory + actor JSON
//! - **Request context** — [`Higgs::from_request`] (feature `ssr`) assembles router,
//!   session actor, and config for Leptos server functions
//! - **Valence accessors** — [`Higgs::valence`] (session actor) and
//!   [`Higgs::unsafe_system_valence`] (operation-tagged `System` actor)
//! - **Subsystem accessors** — feature-gated [`Higgs::chronon`], [`Higgs::boson`],
//!   [`Higgs::photon`] when the host registered backends on the builder
//! - **Startup preflight** — [`preflight`] (with `ssr` / `preflight`) after the
//!   database router is installed
//! - **Permission helpers** — [`server_runtime`] for encode/decode used by
//!   `higgs-macros`-generated server functions
//!
//! # Organized by task
//!
//! | Task | Start here |
//! |------|------------|
//! | Boot config / Valence factory | [`HiggsConfig`], [`HiggsConfig::builder`], [`HiggsValenceFactory`] |
//! | SSR request context | [`Higgs::from_request`] (feature `ssr`) — [example](#ssr--leptos-server-functions) |
//! | Session / system Valence (SSR) | [`Higgs::valence`], [`Higgs::unsafe_system_valence`] |
//! | Call Chronon / Boson / Photon **from SSR** | [`Higgs::chronon`], [`Higgs::boson`], [`Higgs::photon`] |
//! | Valence **inside** Chronon / Boson / Photon workers | [Workers](#workers--chronon--boson--photon) — not `from_request` |
//! | Startup checks | [`preflight`] |
//! | Server `#[server]` wrapper | `higgs-macros::server` (separate crate) |
//! | Session contract / extractors | `higgs-identity`, `higgs-host` |
//!
//! ## Typical host flow
//!
//! 1. Implement a host Valence factory (usually both [`HiggsValenceFactory`] for this
//!    crate and `valence::ValenceFactory` for Chronon / Boson / Photon identity adapters).
//! 2. Build [`HiggsConfig`] once at startup (`valence_factory`, optional `.chronon` /
//!    `.boson` / `.photon`), wrap in `Arc`.
//! 3. Provide `Arc<HiggsConfig>` in Leptos context for server functions.
//! 4. Hand the **same** `valence::ValenceFactory` into Chronon `ContextFactory`, Boson
//!    `ExecutionContextFactory`, and Photon identity / executor wiring (see
//!    [Workers](#workers--chronon--boson--photon)).
//! 5. SSR: [`Higgs::from_request`] → `valence()` / subsystem accessors.
//! 6. Workers: recover Valence from that family's context helper — **not**
//!    [`Higgs::from_request`] (no Leptos request).
//! 7. Optionally run [`preflight`] after the database router is installed and before
//!    schedulers start.
//!
//! # Feature flags
//!
//! | Feature | What it enables |
//! |---------|-----------------|
//! | `ssr` | [`Higgs`], [`Higgs::from_request`], pulls `higgs-host` / Leptos SSR, enables `preflight` |
//! | `preflight` | Re-export of [`preflight`] (also enabled by `ssr`) |
//! | `chronon` | Chronon builder + [`Higgs::chronon`] / scheduler / registry accessors |
//! | `boson` | Boson builder + [`Higgs::boson`] |
//! | `photon` | Photon builder + [`Higgs::photon`] |
//! | `full` | `chronon` + `boson` + `photon` |
//! | `test-utils` | Test doubles from [`higgs_core`] |
//!
//! Default features are empty — enable `ssr` for request context, and subsystem
//! features (or `full`) when the host wires those backends.
//!
//! # Runnable examples
//!
//! `cargo run -p higgs --example config_boot`
//! `cargo run -p higgs --example shared_factory --features ssr`
//! `cargo run -p higgs --example preflight_boot --features preflight`
//! `cargo run -p higgs --example axum_session_host --features ssr`
//! `cargo run -p higgs --example server_fn_context --features ssr`
//! `cargo run -p higgs --example server_fn_backends --features full,ssr`
//! `cargo run -p higgs --example chronon_job --features chronon`
//! `cargo run -p higgs --example boson_task --features boson`
//! `cargo run -p higgs --example photon_worker --features photon`
//!
//! See `higgs/examples/README.md` for the teaching path.
//!
//! # SSR — Leptos server functions
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "ssr", feature = "test-utils"))]
//! # async fn demo() -> Result<(), leptos::prelude::ServerFnError> {
//! use std::sync::Arc;
//! use higgs::{Higgs, HiggsConfig, test_support::UnreachableValenceFactory};
//!
//! let config = Arc::new(
//!     HiggsConfig::builder()
//!         .valence_factory(UnreachableValenceFactory)
//!         // .chronon(scheduler, backend, registry)  // feature = "chronon"
//!         // .photon(photon)                         // feature = "photon"
//!         // .boson(backend)                         // feature = "boson"
//!         .build()
//!         .expect("valence_factory set"),
//! );
//! // Host boot: leptos::provide_context(config.clone());
//!
//! let ctx = Higgs::from_request().await?;
//! let _valence = ctx.valence().map_err(leptos::prelude::ServerFnError::new)?;
//! // With features = ["chronon" | "boson" | "photon"] — call backends from SSR:
//! // let _ = ctx.chronon()?;
//! # let _ = config;
//! # Ok(())
//! # }
//! ```
//!
//! # Workers — Chronon / Boson / Photon
//!
//! Jobs, tasks, and subscribe handlers do **not** use [`Higgs::from_request`]. At boot,
//! pass your host `Arc<dyn valence::ValenceFactory>` into each family's identity adapter;
//! inside the handler, recover Valence from that adapter (captured actor JSON → same
//! factory path SSR uses via [`Higgs::valence`]).
//!
//! Hosts often expose one type that implements both [`HiggsValenceFactory`] (for
//! [`HiggsConfig`]) and `valence::ValenceFactory` (for the adapters below).
//!
//! ## Boot — hand the factory to each runtime
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use chronon_valence_identity::ValenceScriptContextFactory;
//! use boson_valence_identity::ValenceExecutionContextFactory;
//! use photon_valence_identity::ValenceIdentityFactory;
//!
//! let chronon_ctx = Arc::new(ValenceScriptContextFactory::new(Arc::clone(&valence_factory)));
//! let boson_ctx = ValenceExecutionContextFactory::new(Arc::clone(&valence_factory));
//! photon.start_executor(Arc::new(ValenceIdentityFactory::new(Arc::clone(&valence_factory))))?;
//! ```
//!
//! `Higgs::chronon()` / `boson()` / `photon()` remain for **SSR** code that needs to talk
//! to those backends (enqueue a job, publish an event). They do not replace the worker
//! Valence path above.
//!
//! # Notes
//!
//! - Missing `Arc<HiggsConfig>` in Leptos context → [`HiggsError::ConfigNotInContext`].
//! - Subsystem accessors fail with [`HiggsError::SubsystemNotConfigured`] when the
//!   feature is on but the host never called the matching builder method.
//! - Session middleware should insert `higgs_identity::SessionSnapshot` in Axum
//!   extensions; `higgs_host::host_ctx` / [`Higgs::from_request`] read it for the actor.
mod config;
mod error;

#[cfg(feature = "ssr")]
mod context;

pub use config::{HiggsConfig, HiggsConfigBuilder};
pub use error::HiggsError;
pub use higgs_core::{actor_policy, server_runtime, HiggsValenceFactory};

#[cfg(feature = "preflight")]
pub use higgs_core::preflight;

#[cfg(feature = "ssr")]
pub use context::Higgs;

#[cfg(feature = "ssr")]
pub use higgs_host::{current_operation, require_session, with_operation};

#[cfg(any(test, feature = "test-utils"))]
pub use higgs_core::test_support;
