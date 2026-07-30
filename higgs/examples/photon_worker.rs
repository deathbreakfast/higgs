//! Photon subscribe macro: `HiggsConfig` boot + Valence from event actor JSON.
//!
//! Defines `#[topic]` / `#[subscribe]`, boots Photon on `HiggsConfig` with
//! `ValenceIdentityFactory`, publishes a User-scoped event, and rebuilds Valence inside the
//! handler from the transport event (worker path — not `Higgs::from_request`).
//!
//! ```bash
//! cargo run -p higgs --example photon_worker --features photon
//! ```

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::unused_async, clippy::used_underscore_binding)]
#![allow(missing_docs)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use higgs::actor_policy::external_actor_json_policy;
use higgs::{HiggsConfig, HiggsValenceFactory};
use photon::{subscribe, topic, Actor, Event, Photon};
use photon_valence_identity::ValenceIdentityFactory;
use valence::{
    install_default_mem_router, RouterValenceFactory, RouterValenceFactoryConfig, Valence,
    ValenceFactory, DEFAULT_IN_MEMORY_ROUTER_KEY,
};

static HANDLER_OK: AtomicBool = AtomicBool::new(false);
static PROCESS_FACTORY: OnceLock<Arc<dyn ValenceFactory>> = OnceLock::new();

struct MemHiggsFactory(RouterValenceFactory);

impl HiggsValenceFactory for MemHiggsFactory {
    fn build(&self, actor_json: &serde_json::Value) -> anyhow::Result<Valence> {
        self.0.build(actor_json).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

impl ValenceFactory for MemHiggsFactory {
    fn build(&self, actor_json: &serde_json::Value) -> valence::Result<Valence> {
        self.0.build(actor_json)
    }
}

fn mem_factory() -> MemHiggsFactory {
    let router = install_default_mem_router();
    let config = RouterValenceFactoryConfig::new(DEFAULT_IN_MEMORY_ROUTER_KEY)
        .actor_json_policy(external_actor_json_policy());
    MemHiggsFactory(RouterValenceFactory::new(router, config))
}

#[topic(name = "higgs.demo.greeting", keyed_by = "name")]
pub struct HiggsDemoGreeting {
    pub name: String,
    pub message: String,
}

/// Idiomatic Photon handler: rebuild Valence from the published actor JSON (not `from_request`).
#[subscribe(topic = "higgs.demo.greeting", durable = "higgs-demo-logger")]
async fn on_higgs_demo_greeting(
    _actor: Box<dyn Actor>,
    event: HiggsDemoGreeting,
    transport: &Event,
) -> photon::Result<()> {
    let factory = PROCESS_FACTORY
        .get()
        .ok_or_else(|| photon::PhotonError::Internal("valence factory not installed".into()))?;
    let valence = factory
        .build(&transport.actor_json)
        .map_err(|e| photon::PhotonError::Internal(e.to_string()))?;
    if valence.actor().user_id() != Some("photon-worker-user") {
        return Err(photon::PhotonError::Internal(format!(
            "unexpected actor for greeting {}",
            event.name
        )));
    }
    let _ = event.message;
    HANDLER_OK.store(true, Ordering::SeqCst);
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Dev transport key from Photon embedded_mem docs (base64 32-byte key).
    std::env::set_var(
        "PHOTON_TRANSPORT_KEY",
        "cGhvdG9uLWRldi10cmFuc3BvcnQta2V5LTMyYnl0ZXM=",
    );

    let factory = Arc::new(mem_factory());
    let _ = PROCESS_FACTORY.set(Arc::clone(&factory) as Arc<dyn ValenceFactory>);

    let photon = Arc::new(Photon::builder().auto_registry().build()?);
    photon.start_executor(Arc::new(ValenceIdentityFactory::new(
        Arc::clone(&factory) as Arc<dyn ValenceFactory>
    )))?;

    let config = Arc::new(
        HiggsConfig::builder()
            .valence_factory_arc(Arc::clone(&factory) as Arc<dyn HiggsValenceFactory>)
            .photon(Arc::clone(&photon))
            .build()?,
    );

    // SSR-side accessor is configured.
    let _ = config.photon()?;

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let actor_json = serde_json::json!({"User": {"user_id": "photon-worker-user"}});
    let payload = serde_json::to_value(HiggsDemoGreeting {
        name: "world".into(),
        message: "hello from higgs photon_worker".into(),
    })?;
    let _event_id = photon
        .publish("higgs.demo.greeting", Some("world"), actor_json, payload)
        .await?;

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    anyhow::ensure!(
        HANDLER_OK.load(Ordering::SeqCst),
        "#[subscribe] handler must recover Valence and run"
    );

    let system = valence::Actor::System {
        operation: "photon_worker".into(),
    };
    anyhow::ensure!(
        ValenceFactory::build(factory.as_ref(), &serde_json::to_value(&system)?).is_err(),
        "external System actor JSON must be rejected"
    );

    println!("photon_worker: OK — #[subscribe] + Valence from event + HiggsConfig.photon");
    Ok(())
}
