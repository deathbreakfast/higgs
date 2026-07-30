//! Boson task macro: `HiggsConfig` boot + `valence_from_context` inside the handler.
//!
//! Defines a `#[task]`, boots Boson with `ValenceExecutionContextFactory` on `HiggsConfig`,
//! enqueues one job, and runs it via `ManualWorker` (worker path — not `Higgs::from_request`).
//!
//! ```bash
//! cargo run -p higgs --example boson_task --features boson
//! ```

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::expect_used)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use boson_backend_mem::MemQueueBackend;
use boson_coordinator::{BosonCoordinatorBackend, CoordinatorAdapter};
use boson_core::{ExecutionContext, QueueBackend, QueueRouter};
use boson_macros::task;
use boson_runtime::{configure, Boson};
use boson_valence_identity::{
    router_config_reject_external_system, valence_from_context, ValenceExecutionContextFactory,
};
use higgs::{HiggsConfig, HiggsValenceFactory};
use serde_json::json;
use valence::{
    DatabaseRouter, InMemoryBackend, RouterValenceFactory, Valence, ValenceFactory,
    DEFAULT_IN_MEMORY_ROUTER_KEY,
};

static GREET_RUNS: AtomicUsize = AtomicUsize::new(0);

struct MemHiggsFactory(Arc<dyn ValenceFactory>);

impl HiggsValenceFactory for MemHiggsFactory {
    fn build(&self, actor_json: &serde_json::Value) -> anyhow::Result<Valence> {
        self.0.build(actor_json).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

fn mem_valence_factory() -> Arc<dyn ValenceFactory> {
    let mut router = DatabaseRouter::new();
    router.register(
        DEFAULT_IN_MEMORY_ROUTER_KEY.to_string(),
        Arc::new(InMemoryBackend::new()),
    );
    RouterValenceFactory::arc(
        Arc::new(router),
        router_config_reject_external_system(DEFAULT_IN_MEMORY_ROUTER_KEY),
    )
}

/// Idiomatic Boson handler: recover Valence from the execution context (not `from_request`).
#[task(name = "higgs_demo_greet")]
#[allow(clippy::unused_async)]
async fn higgs_demo_greet(ctx: Box<dyn ExecutionContext>, name: String) -> boson_core::Result<()> {
    let valence = valence_from_context(ctx.as_ref())?;
    if valence.actor().user_id() != Some("boson-task-user") {
        return Err(boson_core::BosonError::internal(format!(
            "unexpected actor for greet({name})"
        )));
    }
    GREET_RUNS.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let valence_factory = mem_valence_factory();
    let exec_factory = ValenceExecutionContextFactory::new(Arc::clone(&valence_factory));

    let queue_backend: Arc<dyn QueueBackend> = Arc::new(MemQueueBackend::new());
    QueueRouter::set_global(QueueRouter::with_default(queue_backend));

    let (runtime, manual) = Boson::builder()
        .queue_backend_from_global()
        .execution_context_factory(exec_factory.clone())
        .auto_registry()
        .without_worker()
        .build_manual()?;
    configure(runtime.clone());

    let boson_backend: Arc<dyn BosonCoordinatorBackend> =
        Arc::new(CoordinatorAdapter::new(Arc::new(runtime)));

    let config = Arc::new(
        HiggsConfig::builder()
            .valence_factory(MemHiggsFactory(Arc::clone(&valence_factory)))
            .boson(Arc::clone(&boson_backend))
            .build()?,
    );

    // SSR-side accessor is configured (hosts enqueue from server fns via this handle).
    let _ = config.boson_backend()?;

    HiggsDemoGreet::send_with(
        json!({"User": {"user_id": "boson-task-user"}}),
        HiggsDemoGreetParams {
            name: "world".into(),
        },
    )
    .await?;

    anyhow::ensure!(manual.try_run_next().await, "expected one task to run");
    anyhow::ensure!(GREET_RUNS.load(Ordering::SeqCst) == 1);

    let system = json!({"System": {"operation": "boson_task"}});
    anyhow::ensure!(
        exec_factory.build_valence(&system).is_err(),
        "external System actor JSON must be rejected"
    );

    println!("boson_task: OK — #[task] + valence_from_context + HiggsConfig.boson");
    Ok(())
}
