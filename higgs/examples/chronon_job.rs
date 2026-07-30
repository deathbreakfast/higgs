//! Chronon script macro: `HiggsConfig` boot + `valence_from_context` inside the handler.
//!
//! Defines a `#[chronon_coordinator_macros::script]`, boots Chronon on `HiggsConfig`, and invokes
//! the registered script with `ValenceScriptContextFactory` (worker path — not
//! `Higgs::from_request`).
//!
//! ```bash
//! cargo run -p higgs --example chronon_job --features chronon
//! ```

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chronon_coordinator::{
    snapshot_job_actor_from_valence, validate_external_job_actor_json, ChrononCoordinatorBackend,
    Job, JobRevision, Result as ChrononResult, Run, Scheduler, ScriptRegistry,
};
use chronon_core::{ContextFactory, ScriptContext};
use chronon_valence_identity::{
    router_config_reject_external_system, valence_from_context, ValenceScriptContextFactory,
};
use higgs::{HiggsConfig, HiggsValenceFactory};
use valence::{
    DatabaseRouter, InMemoryBackend, RouterValenceFactory, Valence, ValenceFactory,
    DEFAULT_IN_MEMORY_ROUTER_KEY,
};

/// Host-owned in-memory Chronon backend (teaching only).
#[derive(Default)]
struct LocalBackend {
    jobs: Mutex<Vec<Job>>,
}

impl LocalBackend {
    fn store_job(&self, job: Job) {
        let mut jobs = self.jobs.lock().expect("local backend lock");
        if let Some(existing) = jobs
            .iter_mut()
            .find(|existing| existing.job_id == job.job_id)
        {
            *existing = job;
        } else {
            jobs.push(job);
        }
    }
}

#[async_trait]
impl ChrononCoordinatorBackend for LocalBackend {
    async fn load_jobs_from_db(&self) -> ChrononResult<()> {
        Ok(())
    }

    async fn upsert_job(&self, job: Job) -> ChrononResult<()> {
        validate_external_job_actor_json(&job.actor_json)?;
        self.store_job(job);
        Ok(())
    }

    async fn upsert_job_with_valence(&self, valence: &Valence, mut job: Job) -> ChrononResult<()> {
        snapshot_job_actor_from_valence(&mut job, valence)?;
        self.store_job(job);
        Ok(())
    }

    async fn get_job(&self, job_id: &str) -> Option<Job> {
        self.jobs
            .lock()
            .expect("local backend lock")
            .iter()
            .find(|job| job.job_id == job_id)
            .cloned()
    }

    async fn get_job_by_name(&self, job_name: &str) -> Option<Job> {
        self.jobs
            .lock()
            .expect("local backend lock")
            .iter()
            .find(|job| job.job_name == job_name)
            .cloned()
    }

    async fn list_jobs(&self) -> Vec<Job> {
        self.jobs.lock().expect("local backend lock").clone()
    }

    async fn list_runs(
        &self,
        _job_id: Option<&str>,
        _status: Option<&str>,
        _offset: usize,
        _limit: usize,
    ) -> ChrononResult<Vec<Run>> {
        Ok(Vec::new())
    }

    async fn get_run(&self, _run_id: &str) -> ChrononResult<Option<Run>> {
        Ok(None)
    }

    async fn pause_job(&self, _job_id: &str) -> ChrononResult<()> {
        Ok(())
    }

    async fn resume_job(&self, _job_id: &str) -> ChrononResult<()> {
        Ok(())
    }

    async fn list_revisions(&self, _job_id_or_name: &str) -> ChrononResult<Vec<JobRevision>> {
        Ok(Vec::new())
    }

    async fn update_job_config(&self, _job_id: &str, updated: Job) -> ChrononResult<()> {
        self.upsert_job(updated).await
    }

    async fn update_job_config_with_valence(
        &self,
        valence: &Valence,
        job_id: &str,
        updated: Job,
    ) -> ChrononResult<()> {
        let _ = job_id;
        self.upsert_job_with_valence(valence, updated).await
    }

    async fn run_now(&self, job_id: &str) -> ChrononResult<String> {
        Ok(format!("local-run-{job_id}"))
    }

    async fn run_now_with_params(
        &self,
        job_id: &str,
        _params_override: Option<serde_json::Value>,
    ) -> ChrononResult<String> {
        self.run_now(job_id).await
    }
}

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

/// Idiomatic Chronon handler: recover Valence from the script context (not `from_request`).
#[chronon_coordinator_macros::script(name = "higgs_demo_cleanup")]
#[allow(clippy::unused_async)]
async fn higgs_demo_cleanup(ctx: Box<dyn ScriptContext>) -> anyhow::Result<()> {
    let valence = valence_from_context(&*ctx)?;
    anyhow::ensure!(valence.actor().user_id() == Some("chronon-job-user"));
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let valence_factory = mem_valence_factory();
    let scheduler = Arc::new(Scheduler::from_inventory());
    let registry: Arc<ScriptRegistry> = scheduler.registry_arc();
    let backend: Arc<dyn ChrononCoordinatorBackend> = Arc::new(LocalBackend::default());

    let config = Arc::new(
        HiggsConfig::builder()
            .valence_factory(MemHiggsFactory(Arc::clone(&valence_factory)))
            .chronon(
                Arc::clone(&scheduler),
                Arc::clone(&backend),
                Arc::clone(&registry),
            )
            .build()?,
    );

    // SSR-side accessor is configured (hosts call this from server fns to talk to Chronon).
    let _ = config.chronon_backend()?;

    anyhow::ensure!(
        registry.contains("higgs_demo_cleanup"),
        "#[chronon_coordinator_macros::script] must register via inventory"
    );

    let script_ctx = Arc::new(ValenceScriptContextFactory::new(Arc::clone(
        &valence_factory,
    ))) as Arc<dyn ContextFactory>;
    let actor = serde_json::json!({"User": {"user_id": "chronon-job-user"}});
    let context = script_ctx.build(&actor)?;
    let script = registry
        .get("higgs_demo_cleanup")
        .ok_or_else(|| anyhow::anyhow!("script missing after inventory register"))?;
    (script.invoke)(context, serde_json::json!({}))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let system = serde_json::json!({"System": {"operation": "chronon_job"}});
    anyhow::ensure!(
        ValenceScriptContextFactory::new(Arc::clone(&valence_factory))
            .build_valence(&system)
            .is_err(),
        "external System actor JSON must be rejected"
    );

    println!("chronon_job: OK — #[script] + valence_from_context + HiggsConfig.chronon");
    Ok(())
}
