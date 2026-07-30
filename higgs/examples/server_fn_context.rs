//! Leptos server function: `provide_context` + `Higgs::from_request` → `valence()`.
//!
//! Standalone composition stub — no HTTP server. Provides `Arc<HiggsConfig>` and
//! Axum `Parts` (router + session extensions) in a Leptos `Owner`, then calls a
//! real `#[higgs_macros::server(auth)]` function that uses `from_request`.
//!
//! ```bash
//! cargo run -p higgs --example server_fn_context --features ssr
//! ```

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::expect_used)]
#![allow(missing_docs)]

use std::sync::Arc;

use axum::http::request::Parts;
use axum::http::Request;
use higgs::actor_policy::external_actor_json_policy;
use higgs::{Higgs, HiggsConfig, HiggsValenceFactory};
use higgs_identity::SessionSnapshot;
use leptos::prelude::*;
use valence::{
    install_default_mem_router, RouterValenceFactory, RouterValenceFactoryConfig, Valence,
    ValenceFactory, DEFAULT_IN_MEMORY_ROUTER_KEY,
};

struct MemHiggsFactory(RouterValenceFactory);

impl HiggsValenceFactory for MemHiggsFactory {
    fn build(&self, actor_json: &serde_json::Value) -> anyhow::Result<Valence> {
        self.0.build(actor_json).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

fn mem_factory() -> MemHiggsFactory {
    let router = install_default_mem_router();
    let config = RouterValenceFactoryConfig::new(DEFAULT_IN_MEMORY_ROUTER_KEY)
        .actor_json_policy(external_actor_json_policy());
    MemHiggsFactory(RouterValenceFactory::new(router, config))
}

#[higgs_macros::server(auth)]
pub async fn whoami() -> Result<String, ServerFnError> {
    let ctx = Higgs::from_request().await?;
    let valence = ctx.valence().map_err(ServerFnError::new)?;
    let user = valence
        .actor()
        .user_id()
        .ok_or_else(|| ServerFnError::new("expected User actor"))?;
    Ok(user.to_string())
}

fn request_parts(router: Arc<valence::DatabaseRouter>, session: Option<SessionSnapshot>) -> Parts {
    let mut req = Request::builder().uri("/").body(()).expect("empty request");
    req.extensions_mut().insert(router);
    if let Some(session) = session {
        req.extensions_mut().insert(session);
    }
    let (parts, ()) = req.into_parts();
    parts
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let factory = Arc::new(mem_factory());
    let config = Arc::new(
        HiggsConfig::builder()
            .valence_factory_arc(factory)
            .build()?,
    );
    let db_router = Arc::new(valence::DatabaseRouter::new());

    let owner = Owner::new();
    owner.set();
    provide_context(Arc::clone(&config));
    provide_context(request_parts(
        Arc::clone(&db_router),
        Some(SessionSnapshot::new("user:demo", b"example-auth-hash")),
    ));

    let user = whoami().await.map_err(|e| anyhow::anyhow!("{e}"))?;
    anyhow::ensure!(user == "user:demo");

    println!("server_fn_context: OK — from_request + valence");
    Ok(())
}
