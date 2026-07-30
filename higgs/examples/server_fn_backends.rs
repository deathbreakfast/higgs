//! Leptos server function: `from_request` → Chronon / Boson / Photon accessors.
//!
//! Standalone composition stub — no HTTP server. Provides `Arc<HiggsConfig>` (with all three
//! backends) and Axum `Parts` in a Leptos `Owner`, then calls a `#[higgs_macros::server(auth)]`
//! function that uses `Higgs::from_request` plus `ctx.chronon()` / `boson()` / `photon()`.
//!
//! ```bash
//! cargo run -p higgs --example server_fn_backends --features full,ssr
//! ```

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::expect_used)]
#![allow(missing_docs)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::http::request::Parts;
use axum::http::Request;
use boson_backend_mem::MemQueueBackend;
use boson_coordinator::{BosonCoordinatorBackend, CoordinatorAdapter};
use boson_core::{JsonExecutionContextFactory, QueueBackend, QueueRouter};
use boson_runtime::Boson;
use chronon_coordinator::{
    snapshot_job_actor_from_valence, validate_external_job_actor_json, ChrononCoordinatorBackend,
    Job, JobRevision, Result as ChrononResult, Run, Scheduler, ScriptRegistry,
};
use higgs::actor_policy::external_actor_json_policy;
use higgs::{Higgs, HiggsConfig, HiggsValenceFactory};
use higgs_identity::SessionSnapshot;
use leptos::prelude::*;
use photon::Photon;
use valence::{
    install_default_mem_router, RouterValenceFactory, RouterValenceFactoryConfig, Valence,
    ValenceFactory, DEFAULT_IN_MEMORY_ROUTER_KEY,
};

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
pub async fn ping_backends() -> Result<String, ServerFnError> {
    let ctx = Higgs::from_request().await?;
    let valence = ctx.valence().map_err(ServerFnError::new)?;
    let user = valence
        .actor()
        .user_id()
        .ok_or_else(|| ServerFnError::new("expected User actor"))?;

    let _chronon = ctx.chronon().map_err(ServerFnError::new)?;
    let _boson = ctx.boson().map_err(ServerFnError::new)?;
    let _photon = ctx.photon().map_err(ServerFnError::new)?;

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
    std::env::set_var(
        "PHOTON_TRANSPORT_KEY",
        "cGhvdG9uLWRldi10cmFuc3BvcnQta2V5LTMyYnl0ZXM=",
    );

    let factory = Arc::new(mem_factory());

    let scheduler = Arc::new(Scheduler::from_inventory());
    let registry: Arc<ScriptRegistry> = scheduler.registry_arc();
    let chronon_backend: Arc<dyn ChrononCoordinatorBackend> = Arc::new(LocalBackend::default());

    let queue_backend: Arc<dyn QueueBackend> = Arc::new(MemQueueBackend::new());
    QueueRouter::set_global(QueueRouter::with_default(queue_backend));
    let boson_runtime = Arc::new(
        Boson::builder()
            .queue_backend_from_global()
            .execution_context_factory(JsonExecutionContextFactory)
            .auto_registry()
            .build()?,
    );
    let boson_backend: Arc<dyn BosonCoordinatorBackend> =
        Arc::new(CoordinatorAdapter::new(boson_runtime));

    let photon = Arc::new(Photon::builder().auto_registry().build()?);

    let config = Arc::new(
        HiggsConfig::builder()
            .valence_factory_arc(factory)
            .chronon(scheduler, chronon_backend, registry)
            .boson(boson_backend)
            .photon(photon)
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

    let user = ping_backends().await.map_err(|e| anyhow::anyhow!("{e}"))?;
    anyhow::ensure!(user == "user:demo");

    println!("server_fn_backends: OK — from_request + chronon/boson/photon accessors");
    Ok(())
}
