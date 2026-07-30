//! Axum host: session extension → Higgs context → worker factory rebuild.
//!
//! Production Leptos server functions call `Higgs::from_request()` (Leptos context +
//! `higgs_host::host_ctx`). This example mirrors that assembly on plain Axum: middleware
//! inserts `SessionSnapshot`, the handler builds `HostRequestCtx`, then
//! `Higgs::from_parts` + the shared `HiggsValenceFactory` rebuilds Valence for a worker.
//!
//! ## When to use
//! Teach the identity path hosts must wire before Chronon/Boson/Photon workers.
//!
//! ## Command
//! ```bash
//! CARGO_BUILD_JOBS=1 cargo run -p higgs --example axum_session_host --features ssr
//! ```
//!
//! ## Success
//! Stdout prints `axum_session_host: OK — session → Higgs → worker factory`.
//!
//! ## See also
//! `shared_factory`; [`Higgs::from_parts`](higgs::Higgs::from_parts);
//! [`SessionSnapshot`](higgs_identity::SessionSnapshot).

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use axum::body::Body;
use axum::extract::Extension;
use axum::http::{Request, StatusCode};
use axum::middleware::{from_fn, Next};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use higgs::actor_policy::external_actor_json_policy;
use higgs::{Higgs, HiggsConfig, HiggsValenceFactory};
use higgs_host::HostRequestCtx;
use higgs_identity::SessionSnapshot;
use tower::ServiceExt;
use valence::{
    InMemoryBackend, RouterValenceFactory, RouterValenceFactoryConfig, Valence, ValenceFactory,
    DEFAULT_IN_MEMORY_ROUTER_KEY,
};

struct MemHiggsFactory(RouterValenceFactory);

impl HiggsValenceFactory for MemHiggsFactory {
    fn build(&self, actor_json: &serde_json::Value) -> anyhow::Result<Valence> {
        self.0.build(actor_json).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

fn mem_factory() -> MemHiggsFactory {
    let valence = Valence::builder()
        .add_backend("default", Arc::new(InMemoryBackend::new()))
        .build()
        .expect("mem valence");
    let router = Arc::clone(valence.database_router());
    let config = RouterValenceFactoryConfig::new(DEFAULT_IN_MEMORY_ROUTER_KEY)
        .actor_json_policy(external_actor_json_policy());
    MemHiggsFactory(RouterValenceFactory::new(router, config))
}

/// Lab middleware: `X-Demo-User` → `SessionSnapshot` (hosts use real session cookies).
async fn inject_demo_session(mut req: Request<Body>, next: Next) -> Response {
    let demo_user = req
        .headers()
        .get("x-demo-user")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(str::to_owned);
    if let Some(user) = demo_user {
        req.extensions_mut()
            .insert(SessionSnapshot::new(user, b"example-auth-hash"));
    }
    next.run(req).await
}

async fn whoami_and_enqueue(
    Extension(config): Extension<Arc<HiggsConfig>>,
    Extension(router): Extension<Arc<valence::DatabaseRouter>>,
    session: Option<Extension<SessionSnapshot>>,
) -> Result<String, StatusCode> {
    // Same pieces `Higgs::from_request` gathers inside a Leptos server fn.
    let host = HostRequestCtx {
        database_router: router,
        session: session.map(|Extension(s)| s),
    };
    let ctx = Higgs::from_parts(host, Arc::clone(&config));
    let interactive = ctx
        .valence()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user = interactive
        .actor()
        .user_id()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Worker path: capture actor JSON, rebuild via the shared factory.
    let actor_json =
        serde_json::to_value(ctx.actor()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let worker = config
        .valence_factory()
        .build(&actor_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if worker.actor().user_id() != Some(user) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(format!("ok:{user}"))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let factory = Arc::new(mem_factory());
    let config = Arc::new(
        HiggsConfig::builder()
            .valence_factory_arc(factory)
            .build()?,
    );
    let router = Arc::new(valence::DatabaseRouter::new());

    let app = Router::new()
        .route("/api/whoami", get(whoami_and_enqueue))
        .layer(from_fn(inject_demo_session))
        .layer(Extension(config))
        .layer(Extension(router));

    let anon = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/whoami")
                .body(Body::empty())
                .expect("anon"),
        )
        .await?;
    anyhow::ensure!(anon.status() == StatusCode::UNAUTHORIZED);

    let authed = app
        .oneshot(
            Request::builder()
                .uri("/api/whoami")
                .header("x-demo-user", "user:demo")
                .body(Body::empty())
                .expect("authed"),
        )
        .await?;
    anyhow::ensure!(authed.status() == StatusCode::OK);
    let body = axum::body::to_bytes(authed.into_body(), 1024).await?;
    anyhow::ensure!(body.as_ref() == b"ok:user:demo");

    println!("axum_session_host: OK — session → Higgs → worker factory");
    Ok(())
}
