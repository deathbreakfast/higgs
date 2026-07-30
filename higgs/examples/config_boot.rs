//! Process-wide `HiggsConfig` boot with an in-memory Valence factory.
//!
//! Standalone — no external services.
//!
//! ```bash
//! cargo run -p higgs --example config_boot
//! ```

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::expect_used)]

use higgs::{HiggsConfig, HiggsValenceFactory};
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

fn mem_factory() -> MemHiggsFactory {
    let router = install_default_mem_router();
    MemHiggsFactory(RouterValenceFactory::new(
        router,
        RouterValenceFactoryConfig::new(DEFAULT_IN_MEMORY_ROUTER_KEY),
    ))
}

fn main() -> anyhow::Result<()> {
    let config = HiggsConfig::builder()
        .valence_factory(mem_factory())
        .build()?;

    let anonymous = serde_json::to_value(Actor::Anonymous)?;
    let user = serde_json::to_value(Actor::User {
        user_id: "user:demo".into(),
    })?;

    let v_anon = config.valence_factory().build(&anonymous)?;
    let v_user = config.valence_factory().build(&user)?;

    assert!(matches!(v_anon.actor(), Actor::Anonymous));
    assert_eq!(v_user.actor().user_id(), Some("user:demo"));

    println!("HiggsConfig booted with in-memory Valence factory (anonymous + user OK)");
    Ok(())
}
