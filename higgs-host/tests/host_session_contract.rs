//! Integration tests for host request session → actor mapping and operation TLS.
//!
//! Covers the public `HostRequestCtx` / `with_operation` contracts that middleware
//! and `higgs` rely on (no live Axum extract / Leptos request needed).

use std::sync::Arc;

use higgs_host::{current_operation, with_operation, HostRequestCtx};
use higgs_identity::SessionSnapshot;
use valence::{Actor, DatabaseRouter};

#[test]
fn authenticated_session_maps_to_user_actor_happy_path() {
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

#[tokio::test]
async fn with_operation_nested_scopes_happy_path() {
    // Inner scope must not leak; after nest completes, outer value remains until exit.
    let outer = with_operation("outer", async {
        assert_eq!(current_operation(), Some("outer"));
        let inner = with_operation("inner", async { current_operation() }).await;
        assert_eq!(inner, Some("inner"));
        current_operation()
    })
    .await;
    assert_eq!(outer, Some("outer"));
    assert!(current_operation().is_none());
}
