//! Per-request platform Higgs context.

use std::sync::Arc;

use higgs_host::HostRequestCtx;
use higgs_identity::SessionUserId;
use leptos::prelude::*;

use crate::config::HiggsConfig;
use crate::HiggsError;

/// Per-request application context for server functions.
///
/// Available on crate feature `ssr`. Construct via [`Higgs::from_request`]; see the
/// [crate root](crate) for boot wiring and a quick example.
pub struct Higgs {
    inner: higgs_core::Higgs,
    config: Arc<HiggsConfig>,
}

impl Higgs {
    /// Assemble context from host request parts and config (tests / wrappers).
    #[must_use]
    pub fn from_parts(host: HostRequestCtx, config: Arc<HiggsConfig>) -> Self {
        let inner = higgs_core::Higgs::from_parts(host, config.core_arc());
        Self { inner, config }
    }

    /// Assemble context from the current request (router, session snapshot, platform config).
    ///
    /// Requires `Arc<HiggsConfig>` in Leptos context and Axum extensions for the Valence
    /// router (and optional `higgs_identity::SessionSnapshot`). See the [SSR
    /// example](crate#ssr--leptos-server-functions). For Chronon / Boson / Photon
    /// handlers, see [Workers](crate#workers--chronon--boson--photon) instead.
    ///
    /// # Errors
    ///
    /// - [`HiggsError::ConfigNotInContext`] when `provide_context(Arc<HiggsConfig>)` was
    ///   not called at host boot (wrapped in `ServerFnError`)
    /// - `ServerFnError` from [`higgs_host::host_ctx`] when the database router extension is
    ///   missing or extraction fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "ssr")]
    /// # async fn demo() -> Result<(), leptos::prelude::ServerFnError> {
    /// use higgs::Higgs;
    ///
    /// let ctx = Higgs::from_request().await?;
    /// let valence = ctx.valence().map_err(leptos::prelude::ServerFnError::new)?;
    /// let _ = valence;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_request() -> Result<Self, ServerFnError> {
        let config = use_context::<Arc<HiggsConfig>>()
            .ok_or_else(|| ServerFnError::new(HiggsError::ConfigNotInContext.to_string()))?;

        let host: HostRequestCtx = higgs_host::host_ctx().await?;
        Ok(Self::from_parts(host, config))
    }

    /// The underlying host request context.
    pub const fn host(&self) -> &HostRequestCtx {
        self.inner.host()
    }

    /// Raw Valence [`valence::DatabaseRouter`] without session-scoped privacy.
    ///
    /// Prefer [`Self::valence`] for user-driven CRUD.
    pub const fn unsafe_database_router(&self) -> &Arc<valence::DatabaseRouter> {
        self.inner.unsafe_database_router()
    }

    /// Deprecated alias for [`Self::unsafe_database_router`].
    #[deprecated(
        note = "use unsafe_database_router — prefer valence() for user-driven data access"
    )]
    pub const fn database_router(&self) -> &Arc<valence::DatabaseRouter> {
        self.unsafe_database_router()
    }

    /// Authenticated session user id when present.
    pub const fn session_user_id(&self) -> Option<&SessionUserId> {
        self.inner.session_user_id()
    }

    /// The current request's Valence actor.
    pub fn actor(&self) -> valence::Actor {
        self.inner.actor()
    }

    /// The process-wide [`HiggsConfig`].
    pub fn config(&self) -> &HiggsConfig {
        &self.config
    }

    /// Build Valence via the host-configured factory and current request actor.
    ///
    /// Runnable: `cargo run -p higgs --example server_fn_context --features ssr`
    ///
    /// # Errors
    ///
    /// Returns [`HiggsError::Internal`] if the actor cannot be serialized or the
    /// [`HiggsValenceFactory`](crate::HiggsValenceFactory) fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "ssr")]
    /// # async fn demo(ctx: &higgs::Higgs) -> Result<(), leptos::prelude::ServerFnError> {
    /// let valence = ctx.valence().map_err(leptos::prelude::ServerFnError::new)?;
    /// let _ = valence.actor();
    /// # Ok(())
    /// # }
    /// ```
    pub fn valence(&self) -> Result<valence::Valence, HiggsError> {
        self.inner.valence().map_err(HiggsError::from)
    }

    /// Build Valence scoped to a `System` actor for the current operation.
    ///
    /// **Do not use for user-driven work.** This mints [`valence::Actor::System`], which
    /// bypasses Valence privacy policies. Prefer [`Self::valence`] (session / Anonymous)
    /// after [`higgs_host::require_session`] or `#[higgs_macros::server(auth)]`.
    ///
    /// Uses `higgs_host::current_operation` when set (e.g. by `higgs_macros::server`);
    /// otherwise the operation name is `"<unknown>"`. Reserved for intentional
    /// control-plane operations inside the host after local authorization.
    ///
    /// # Errors
    ///
    /// Returns [`HiggsError::Internal`] on actor serialization or factory failure
    /// (client-facing Display stays opaque).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Prefer with_operation / #[higgs_macros::server] so the System actor is attributed.
    /// let valence = ctx.unsafe_system_valence()?;
    /// let _ = valence.actor();
    /// ```
    pub fn unsafe_system_valence(&self) -> Result<valence::Valence, HiggsError> {
        self.inner.unsafe_system_valence().map_err(HiggsError::from)
    }

    /// The Chronon scheduler.
    ///
    /// Available on crate feature `chronon`.
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if `.chronon(...)` was not called on the
    /// config builder.
    #[cfg(feature = "chronon")]
    pub fn scheduler(&self) -> Result<&chronon_coordinator::Scheduler, HiggsError> {
        self.config.scheduler()
    }

    /// The Chronon coordinator backend.
    ///
    /// Available on crate feature `chronon`.
    /// Runnable (SSR accessors): `cargo run -p higgs --example server_fn_backends --features full,ssr`
    /// Runnable (worker `#[script]`): `cargo run -p higgs --example chronon_job --features chronon`
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if Chronon was not configured at boot.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // SSR: enqueue / inspect after HiggsConfig::builder().chronon(...).build()
    /// let backend = ctx.chronon()?;
    /// let _ = backend;
    /// ```
    #[cfg(feature = "chronon")]
    pub fn chronon(
        &self,
    ) -> Result<&dyn chronon_coordinator::ChrononCoordinatorBackend, HiggsError> {
        self.config.chronon_backend()
    }

    /// The Chronon script registry.
    ///
    /// Available on crate feature `chronon`.
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if Chronon was not configured at boot.
    #[cfg(feature = "chronon")]
    pub fn script_registry(&self) -> Result<&chronon_coordinator::ScriptRegistry, HiggsError> {
        self.config.script_registry()
    }

    /// Shared Chronon backend arc.
    ///
    /// Available on crate feature `chronon`.
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if Chronon was not configured at boot.
    #[cfg(feature = "chronon")]
    pub fn chronon_backend_arc(
        &self,
    ) -> Result<Arc<dyn chronon_coordinator::ChrononCoordinatorBackend>, HiggsError> {
        self.config.chronon_backend_arc()
    }

    /// The Photon pub/sub backend.
    ///
    /// Available on crate feature `photon`.
    /// Runnable (SSR accessors): `cargo run -p higgs --example server_fn_backends --features full,ssr`
    /// Runnable (worker `#[subscribe]`): `cargo run -p higgs --example photon_worker --features photon`
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if `.photon(...)` was not called on the
    /// config builder.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // SSR: publish / subscribe after HiggsConfig::builder().photon(...).build()
    /// let photon = ctx.photon()?;
    /// let _ = photon;
    /// ```
    #[cfg(feature = "photon")]
    pub fn photon(&self) -> Result<&photon::Photon, HiggsError> {
        self.config.photon()
    }

    /// The Boson work engine backend.
    ///
    /// Available on crate feature `boson`.
    /// Runnable (SSR accessors): `cargo run -p higgs --example server_fn_backends --features full,ssr`
    /// Runnable (worker `#[task]`): `cargo run -p higgs --example boson_task --features boson`
    ///
    /// Authorize in the server function before enqueue/publish.
    /// Prefer server-chosen actor JSON (session / Anonymous / fixed System operation).
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if `.boson(...)` was not called on the
    /// config builder.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // SSR: enqueue after HiggsConfig::builder().boson(...).build()
    /// let backend = ctx.boson()?;
    /// let _ = backend;
    /// ```
    #[cfg(feature = "boson")]
    pub fn boson(&self) -> Result<&dyn boson_coordinator::BosonCoordinatorBackend, HiggsError> {
        self.config.boson_backend()
    }
}
