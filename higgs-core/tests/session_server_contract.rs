//! Integration tests for session → Valence and server-runtime permission contracts.
//!
//! Exercises `Higgs::from_parts` / `valence` / `unsafe_system_valence`, config builder, and
//! `server_runtime` encode/decode from outside the crate (host session path).

use std::sync::Arc;

use higgs_core::server_runtime::{
    parse_permission_error_payload, permission_check_failed_payload, permission_denied_payload,
    PermissionErrorPayload, PERMISSION_DENIED_PREFIX,
};
use higgs_core::{Higgs, HiggsConfig, HiggsError, HiggsValenceFactory};
use higgs_host::{with_operation, HostRequestCtx};
use higgs_identity::SessionSnapshot;
use valence::{
    Actor, DatabaseRouter, InMemoryBackend, RouterValenceFactory, RouterValenceFactoryConfig,
    DEFAULT_IN_MEMORY_ROUTER_KEY,
};

fn mem_higgs_factory() -> Arc<dyn HiggsValenceFactory> {
    let mut router = DatabaseRouter::new();
    router.register(
        DEFAULT_IN_MEMORY_ROUTER_KEY.to_string(),
        Arc::new(InMemoryBackend::new()),
    );
    let inner = RouterValenceFactory::arc(
        Arc::new(router),
        RouterValenceFactoryConfig::new(DEFAULT_IN_MEMORY_ROUTER_KEY),
    );
    Arc::new(WrapFactory(inner))
}

struct WrapFactory(Arc<dyn valence::ValenceFactory>);

impl HiggsValenceFactory for WrapFactory {
    fn build(&self, actor_json: &serde_json::Value) -> anyhow::Result<valence::Valence> {
        self.0
            .build(actor_json)
            .map_err(|e| anyhow::anyhow!("valence factory: {e}"))
    }
}

struct FailFactory;

impl HiggsValenceFactory for FailFactory {
    fn build(&self, _actor_json: &serde_json::Value) -> anyhow::Result<valence::Valence> {
        anyhow::bail!("integ factory build failed")
    }
}

fn host_with_session(user_id: &str) -> HostRequestCtx {
    HostRequestCtx {
        database_router: Arc::new(DatabaseRouter::new()),
        session: Some(SessionSnapshot::new(user_id, b"hash")),
    }
}

#[test]
fn config_builder_with_factory_happy_path() {
    let config = HiggsConfig::builder()
        .valence_factory_arc(mem_higgs_factory())
        .build()
        .expect("factory set");
    let _ = config.valence_factory();
}

#[test]
fn config_builder_missing_factory_sad() {
    match HiggsConfig::builder().build() {
        Err(HiggsError::Internal) => {}
        Err(e) => panic!("expected Internal missing factory, got {e}"),
        Ok(_) => panic!("expected error, got Ok"),
    }
}

#[test]
fn higgs_from_parts_session_valence_happy_path() {
    let config = Arc::new(
        HiggsConfig::builder()
            .valence_factory_arc(mem_higgs_factory())
            .build()
            .expect("factory set"),
    );
    let higgs = Higgs::from_parts(host_with_session("user:integ"), config);
    assert_eq!(
        higgs.session_user_id().map(String::as_str),
        Some("user:integ")
    );
    match higgs.actor() {
        Actor::User { user_id } => assert_eq!(user_id, "user:integ"),
        other => panic!("expected User, got {other:?}"),
    }
    let valence = higgs.valence().expect("valence");
    let _ = valence.database_router();
}

#[test]
fn higgs_valence_factory_failure_sad() {
    let config = Arc::new(
        HiggsConfig::builder()
            .valence_factory(FailFactory)
            .build()
            .expect("factory set"),
    );
    let higgs = Higgs::from_parts(host_with_session("user:x"), config);
    match higgs.valence() {
        Err(HiggsError::Internal) => {
            assert_eq!(
                higgs.valence().unwrap_err().to_string(),
                "internal higgs error"
            );
        }
        Err(e) => panic!("expected Internal, got {e}"),
        Ok(_) => panic!("expected error, got Ok"),
    }
}

#[tokio::test]
async fn system_valence_with_operation_happy_path() {
    let config = Arc::new(
        HiggsConfig::builder()
            .valence_factory_arc(mem_higgs_factory())
            .build()
            .expect("factory set"),
    );
    let host = HostRequestCtx {
        database_router: Arc::new(DatabaseRouter::new()),
        session: None,
    };
    let higgs = Higgs::from_parts(host, config);
    let valence = with_operation("ops.seed", async {
        higgs.unsafe_system_valence().expect("system valence")
    })
    .await;
    let _ = valence.database_router();
}

#[test]
fn server_runtime_permission_denied_roundtrip_happy_path() {
    let msg = permission_denied_payload("gauge.admin");
    assert_eq!(
        parse_permission_error_payload(&msg),
        Some(PermissionErrorPayload::Denied {
            permission: "gauge.admin".into(),
        })
    );
}

#[test]
fn server_runtime_permission_check_failed_omits_details_happy_path() {
    let msg = permission_check_failed_payload("gauge.admin", "timeout secret=xyz");
    assert_eq!(msg, "permission_check_failed::gauge.admin");
    assert!(!msg.contains("secret"));
    assert!(!msg.contains("timeout"));
    assert_eq!(
        parse_permission_error_payload(&msg),
        Some(PermissionErrorPayload::CheckFailed {
            permission: "gauge.admin".into(),
            details: String::new(),
        })
    );
}

#[test]
fn server_runtime_parse_rejects_garbage_sad() {
    assert!(parse_permission_error_payload("plain error").is_none());
    assert!(parse_permission_error_payload(PERMISSION_DENIED_PREFIX).is_none());
    assert!(parse_permission_error_payload("permission_check_failed::").is_none());
}
