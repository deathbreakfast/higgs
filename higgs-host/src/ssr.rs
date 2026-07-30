//! SSR helpers: extract Valence router + optional [`higgs_identity::SessionSnapshot`].

use axum::extract::Extension;
use higgs_identity::SessionSnapshot;
use leptos::prelude::ServerFnError;
use leptos_axum::extract;
use std::sync::Arc;
use valence::{Actor, DatabaseRouter};

tokio::task_local! {
    static CURRENT_OPERATION: Option<&'static str>;
}

/// Database plane extracted from Axum extensions (no auth).
///
/// Escape hatch for boot/control-plane paths. Prefer session-scoped Valence via
/// `higgs::Higgs::valence` for user-driven data access.
///
/// # Examples
///
/// ```rust,ignore
/// use higgs_host::unsafe_data_plane;
///
/// let plane = unsafe_data_plane().await?;
/// let _router = &plane.database_router;
/// ```
#[derive(Clone)]
pub struct DataPlaneCtx {
    /// Valence database router for the current request.
    pub database_router: Arc<DatabaseRouter>,
}

/// Full host request context including optional session snapshot.
///
/// # Examples
///
/// ```rust,ignore
/// use higgs_host::{host_ctx, HostRequestCtx};
///
/// let host: HostRequestCtx = host_ctx().await?;
/// let _actor = host.actor();
/// ```
#[derive(Clone)]
pub struct HostRequestCtx {
    /// Valence database router for the current request.
    pub database_router: Arc<DatabaseRouter>,
    /// Session snapshot when auth middleware inserted one; `None` for anonymous requests.
    pub session: Option<SessionSnapshot>,
}

impl HostRequestCtx {
    /// The authenticated session's user id, if any.
    pub fn session_user_id(&self) -> Option<&str> {
        self.session.as_ref().map(|s| s.user_id.as_str())
    }

    /// Whether this request carries a session snapshot (i.e. is authenticated).
    pub const fn is_authenticated(&self) -> bool {
        self.session.is_some()
    }

    /// Derive a Valence [`Actor`] from the session (`User`) or lack thereof (`Anonymous`).
    pub fn actor(&self) -> Actor {
        self.session
            .as_ref()
            .map_or(Actor::Anonymous, |s| Actor::User {
                user_id: s.user_id.clone(),
            })
    }
}

/// Extract the Valence router only (no session).
///
/// **Escape hatch** — bypasses actor-scoped Valence privacy. Prefer
/// `higgs::Higgs::valence` for user-driven CRUD.
///
/// Available on crate feature `ssr`.
///
/// # Errors
///
/// Returns `ServerFnError` when Axum extensions do not contain
/// `Extension<Arc<DatabaseRouter>>` or extraction otherwise fails.
///
/// # Examples
///
/// ```rust,ignore
/// use higgs_host::unsafe_data_plane;
///
/// let plane = unsafe_data_plane().await?;
/// let _ = plane.database_router;
/// ```
pub async fn unsafe_data_plane() -> Result<DataPlaneCtx, ServerFnError> {
    let Extension(database_router): Extension<Arc<DatabaseRouter>> = extract().await?;
    Ok(DataPlaneCtx { database_router })
}

/// Deprecated alias for [`unsafe_data_plane`].
///
/// # Errors
///
/// Same as [`unsafe_data_plane`]: returns `ServerFnError` when Axum extensions do not
/// contain `Extension<Arc<DatabaseRouter>>` or extraction otherwise fails.
#[deprecated(note = "use unsafe_data_plane — prefer Higgs::valence for user-driven data access")]
pub async fn data_plane() -> Result<DataPlaneCtx, ServerFnError> {
    unsafe_data_plane().await
}

/// Extract the Valence router and optional [`SessionSnapshot`] extension.
///
/// Auth middleware in `lepton-host-adapter` should insert `Extension<SessionSnapshot>`
/// when a user is authenticated. Available on crate feature `ssr`.
///
/// Runnable (full host): `cargo run -p higgs --example axum_session_host --features ssr`
///
/// # Errors
///
/// Returns `ServerFnError` when the Valence `DatabaseRouter` extension is missing or
/// extraction otherwise fails. A missing session extension is not an error — `session`
/// is then `None` (anonymous actor).
///
/// # Examples
///
/// ```rust,ignore
/// use higgs_host::{host_ctx, HostRequestCtx};
///
/// let host: HostRequestCtx = host_ctx().await?;
/// let _actor = host.actor();
/// ```
pub async fn host_ctx() -> Result<HostRequestCtx, ServerFnError> {
    let Extension(database_router): Extension<Arc<DatabaseRouter>> = extract().await?;
    let session: Option<Extension<SessionSnapshot>> = extract().await.ok();
    Ok(HostRequestCtx {
        database_router,
        session: session.map(|Extension(s)| s),
    })
}

/// Require an authenticated [`SessionSnapshot`] on the current request.
///
/// Used by `#[higgs_macros::server(auth)]`. Loads session identity only —
/// product user models stay in the host identity adapter.
/// only the session extension inserted by host middleware.
///
/// # Errors
///
/// Missing router extension, or no session (anonymous).
pub async fn require_session() -> Result<SessionSnapshot, ServerFnError> {
    let ctx = host_ctx().await?;
    ctx.session
        .ok_or_else(|| ServerFnError::Args("You must be signed in".into()))
}

/// The current task-local operation name set by [`with_operation`], if any.
///
/// # Examples
///
/// ```rust
/// # async fn demo() {
/// use higgs_host::{current_operation, with_operation};
///
/// assert!(current_operation().is_none());
/// let seen = with_operation("ops.create", async { current_operation() }).await;
/// assert_eq!(seen, Some("ops.create"));
/// # }
/// ```
pub fn current_operation() -> Option<&'static str> {
    CURRENT_OPERATION.try_with(|op| *op).ok().flatten()
}

/// Run `fut` with `operation` set as the current task-local operation name (readable via
/// [`current_operation`] for the duration of `fut`).
///
/// # Examples
///
/// ```rust
/// # async fn demo() {
/// use higgs_host::{current_operation, with_operation};
///
/// let seen = with_operation("ops.create", async { current_operation() }).await;
/// assert_eq!(seen, Some("ops.create"));
/// # }
/// ```
pub async fn with_operation<F, R>(operation: &'static str, fut: F) -> R
where
    F: std::future::Future<Output = R>,
{
    CURRENT_OPERATION.scope(Some(operation), fut).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use valence::DatabaseRouter;

    #[test]
    fn session_snapshot_maps_to_user_actor_happy_path() {
        let ctx = HostRequestCtx {
            database_router: Arc::new(DatabaseRouter::new()),
            session: Some(SessionSnapshot::new("user:alice", b"hash")),
        };
        match ctx.actor() {
            Actor::User { user_id } => assert_eq!(user_id, "user:alice"),
            other => panic!("expected User actor, got {other:?}"),
        }
        assert_eq!(ctx.session_user_id(), Some("user:alice"));
        assert!(ctx.is_authenticated());
    }

    #[test]
    fn missing_session_maps_to_anonymous_actor_happy_path() {
        let ctx = HostRequestCtx {
            database_router: Arc::new(DatabaseRouter::new()),
            session: None,
        };
        assert!(matches!(ctx.actor(), Actor::Anonymous));
        assert!(ctx.session_user_id().is_none());
        assert!(!ctx.is_authenticated());
    }

    #[tokio::test]
    async fn with_operation_sets_task_local_happy_path() {
        assert!(current_operation().is_none());
        let seen = with_operation("ops.create", async { current_operation() }).await;
        assert_eq!(seen, Some("ops.create"));
        assert!(current_operation().is_none());
    }
}
