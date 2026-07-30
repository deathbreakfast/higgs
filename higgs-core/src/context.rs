use std::sync::Arc;

use higgs_host::HostRequestCtx;
use higgs_identity::SessionUserId;
use leptos::prelude::*;

use crate::config::HiggsConfig;
use crate::HiggsError;

/// Per-request application context for server functions.
///
/// Available on crate feature `ssr`. See the [crate-root quick example](crate#quick-example).
///
/// # Examples
///
/// ```rust,no_run
/// # #[cfg(feature = "ssr")]
/// # async fn demo() -> Result<(), leptos::prelude::ServerFnError> {
/// use higgs_core::Higgs;
///
/// let ctx = Higgs::from_request().await?;
/// let valence = ctx.valence().map_err(leptos::prelude::ServerFnError::new)?;
/// let _ = valence;
/// # Ok(())
/// # }
/// ```
pub struct Higgs {
    host: HostRequestCtx,
    session_user_id: Option<SessionUserId>,
    config: Arc<HiggsConfig>,
}

impl Higgs {
    /// Assemble context from an already-extracted host request and config.
    ///
    /// Prefer [`Self::from_request`] in server functions. Platform `higgs` uses this
    /// when wrapping core context with subsystem config.
    #[must_use]
    pub fn from_parts(host: HostRequestCtx, config: Arc<HiggsConfig>) -> Self {
        let session_user_id = host.session_user_id().map(str::to_string);
        Self {
            host,
            session_user_id,
            config,
        }
    }

    /// Assemble context from the current request (router, session snapshot).
    ///
    /// Requires `Arc<HiggsConfig>` in Leptos context. See the [crate-root quick
    /// example](crate#quick-example).
    ///
    /// Runnable: `cargo run -p higgs --example server_fn_context --features ssr`
    ///
    /// # Errors
    ///
    /// - [`HiggsError::ConfigNotInContext`] when config was not provided (wrapped in
    ///   `ServerFnError`)
    /// - `ServerFnError` from [`higgs_host::host_ctx`] when the database router extension is
    ///   missing or extraction fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "ssr")]
    /// # async fn demo() -> Result<(), leptos::prelude::ServerFnError> {
    /// use higgs_core::Higgs;
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

    /// The underlying host request context (router, session snapshot).
    pub const fn host(&self) -> &HostRequestCtx {
        &self.host
    }

    /// Raw Valence [`valence::DatabaseRouter`] without session-scoped privacy.
    ///
    /// Prefer [`Self::valence`] for user-driven CRUD. Boot/control-plane only.
    pub const fn unsafe_database_router(&self) -> &Arc<valence::DatabaseRouter> {
        &self.host.database_router
    }

    /// Deprecated alias for [`Self::unsafe_database_router`].
    #[deprecated(
        note = "use unsafe_database_router — prefer valence() for user-driven data access"
    )]
    pub const fn database_router(&self) -> &Arc<valence::DatabaseRouter> {
        self.unsafe_database_router()
    }

    /// Authenticated session user id when middleware inserted a [`higgs_identity::SessionSnapshot`].
    pub const fn session_user_id(&self) -> Option<&SessionUserId> {
        self.session_user_id.as_ref()
    }

    /// The current request's Valence actor (derived from the session, or `Anonymous`).
    pub fn actor(&self) -> valence::Actor {
        self.host.actor()
    }

    /// The process-wide [`HiggsConfig`] this context was built against.
    pub fn config(&self) -> &HiggsConfig {
        &self.config
    }

    /// Build Valence via the host-configured factory and current request actor.
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
    /// # async fn demo(ctx: &higgs_core::Higgs) -> Result<(), leptos::prelude::ServerFnError> {
    /// let valence = ctx.valence().map_err(leptos::prelude::ServerFnError::new)?;
    /// let _ = valence.actor();
    /// # Ok(())
    /// # }
    /// ```
    pub fn valence(&self) -> Result<valence::Valence, HiggsError> {
        let actor_json = serde_json::to_value(self.host.actor())
            .map_err(|e| HiggsError::internal(format!("actor serialize: {e}")))?;
        self.config
            .valence_factory()
            .build(&actor_json)
            .map_err(|e| HiggsError::internal(e.to_string()))
    }

    /// Build Valence scoped to a `System` actor for the current operation.
    ///
    /// **Do not use for user-driven work.** This mints [`valence::Actor::System`], which
    /// bypasses Valence privacy policies. Prefer [`Self::valence`] (session / Anonymous)
    /// after [`higgs_host::require_session`] or `#[higgs_macros::server(auth)]`.
    ///
    /// Reserved for intentional control-plane operations inside the host after local
    /// authorization. Callers that reach for this to “make a query work” are almost
    /// always missing a privacy policy or auth gate instead.
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
        let operation = higgs_host::current_operation()
            .unwrap_or("<unknown>")
            .to_string();
        log::debug!(
            target: "higgs",
            "unsafe_system_valence operation={operation} session={:?}",
            self.session_user_id
        );
        let actor = valence::Actor::System { operation };
        let actor_json = serde_json::to_value(actor)
            .map_err(|e| HiggsError::internal(format!("actor serialize: {e}")))?;
        self.config
            .valence_factory()
            .build(&actor_json)
            .map_err(|e| HiggsError::internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct CaptureFactory {
        last_actor: Mutex<Option<serde_json::Value>>,
    }

    impl crate::HiggsValenceFactory for CaptureFactory {
        fn build(&self, actor_json: &serde_json::Value) -> anyhow::Result<valence::Valence> {
            *self
                .last_actor
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(actor_json.clone());
            anyhow::bail!("capture-only factory")
        }
    }

    fn test_higgs(factory: Arc<CaptureFactory>) -> Higgs {
        let config = Arc::new(
            HiggsConfig::builder()
                .valence_factory_arc(factory)
                .build()
                .expect("factory set"),
        );
        let host = HostRequestCtx {
            database_router: Arc::new(valence::DatabaseRouter::new()),
            session: None,
        };
        Higgs::from_parts(host, config)
    }

    #[tokio::test]
    async fn unsafe_system_valence_with_operation_happy_path() {
        let factory = Arc::new(CaptureFactory {
            last_actor: Mutex::new(None),
        });
        let higgs = test_higgs(factory.clone());

        let () = higgs_host::with_operation("ops.create", async {
            let err = higgs.unsafe_system_valence().expect_err("capture factory");
            assert!(matches!(err, HiggsError::Internal));
        })
        .await;

        let actor = factory
            .last_actor
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .expect("factory invoked");
        let parsed: valence::Actor =
            serde_json::from_value(actor).expect("actor JSON deserializes");
        match parsed {
            valence::Actor::System { operation } => assert_eq!(operation, "ops.create"),
            other => panic!("expected System actor, got {other:?}"),
        }
    }

    #[test]
    fn unsafe_database_router_matches_host_happy_path() {
        let factory = Arc::new(CaptureFactory {
            last_actor: Mutex::new(None),
        });
        let higgs = test_higgs(factory);
        let ptr = Arc::as_ptr(higgs.unsafe_database_router());
        assert_eq!(ptr, Arc::as_ptr(&higgs.host().database_router));
    }
}
