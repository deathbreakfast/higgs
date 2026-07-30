//! Shared Valence factory for interactive request context and worker rebuild.
//!
//! Standalone, same process — teaches the identity path without a Leptos/Axum host.
//! Production SSR uses `Higgs::from_request` after the host provides `Arc<HiggsConfig>`
//! in Leptos context and session middleware populates `SessionSnapshot` (e.g.
//! `lepton-host-adapter`). Auth is a host responsibility.
//!
//! ```bash
//! cargo run -p higgs --example shared_factory --features ssr
//! ```

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use higgs::actor_policy::external_actor_json_policy;
use higgs::{Higgs, HiggsConfig, HiggsValenceFactory};
use higgs_host::HostRequestCtx;
use higgs_identity::SessionSnapshot;
use valence::{
    install_default_mem_router, Actor, RouterValenceFactory, RouterValenceFactoryConfig, Valence,
    ValenceFactory, DEFAULT_IN_MEMORY_ROUTER_KEY,
};

/// Maps Valence's [`RouterValenceFactory`] to Higgs's [`HiggsValenceFactory`].
struct MemHiggsFactory(RouterValenceFactory);

impl HiggsValenceFactory for MemHiggsFactory {
    fn build(&self, actor_json: &serde_json::Value) -> anyhow::Result<Valence> {
        self.0.build(actor_json).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

fn mem_factory_with_external_policy() -> MemHiggsFactory {
    let router = install_default_mem_router();
    let config = RouterValenceFactoryConfig::new(DEFAULT_IN_MEMORY_ROUTER_KEY)
        .actor_json_policy(external_actor_json_policy());
    MemHiggsFactory(RouterValenceFactory::new(router, config))
}

fn main() -> anyhow::Result<()> {
    let factory = Arc::new(mem_factory_with_external_policy());
    let config = Arc::new(
        HiggsConfig::builder()
            .valence_factory_arc(factory)
            .build()?,
    );

    // Interactive path: assemble context the way tests/wrappers do (`from_parts`).
    let router = Arc::new(valence::DatabaseRouter::new());
    let host = HostRequestCtx {
        database_router: router,
        session: Some(SessionSnapshot::new("user:demo", b"example-auth-hash")),
    };
    let ctx = Higgs::from_parts(host, Arc::clone(&config));
    let interactive = ctx.valence()?;
    assert_eq!(interactive.actor().user_id(), Some("user:demo"));

    // Worker path: rebuild Valence from captured actor JSON via the same factory.
    let actor_json = serde_json::to_value(ctx.actor())?;
    let worker = config.valence_factory().build(&actor_json)?;
    assert_eq!(worker.actor().user_id(), Some("user:demo"));

    // External System JSON must fail closed when the policy is installed.
    let system = serde_json::to_value(Actor::System {
        operation: "demo.op".into(),
    })?;
    let rejected = config.valence_factory().build(&system);
    assert!(
        rejected.is_err(),
        "external System actor JSON must be rejected"
    );

    println!("shared factory: interactive + worker rebuild OK (external System rejected)");
    Ok(())
}
