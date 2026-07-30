# Higgs examples

Canonical teaching path (start here). Topology docs in the `higgs` crate rustdoc
(`cargo doc -p higgs --open`): process-wide config, shared factory, and startup
preflight sections on the crate root.

These examples belong to the `higgs` package. Run them from the
workspace root. Local Valence wiring uses **in-memory** storage (no external
services). SQLite / remote fleets are host-owned — Higgs does not encode that
topology.

## 1. Process-wide config — `config_boot` (standalone)

Boot `HiggsConfig` with a mem-backed `HiggsValenceFactory` (wraps Valence
`RouterValenceFactory`). Builds Anonymous and User Valence instances.

```bash
cargo run -p higgs --example config_boot
```

Success: stdout prints `HiggsConfig booted with in-memory Valence factory (anonymous + user OK)`.

## 2. Shared factory (interactive + worker) — `shared_factory` (standalone)

Same factory Arc for a simulated request (`Higgs::from_parts` + session) and a worker
rebuild from captured actor JSON. Installs `external_actor_json_policy()` so external
System JSON fails closed.

Production SSR uses `Higgs::from_request` after the host provides `Arc<HiggsConfig>` in
Leptos context and session middleware populates `SessionSnapshot` (e.g.
`lepton-host-adapter`). Auth is a host responsibility.

```bash
cargo run -p higgs --example shared_factory --features ssr
```

Success: stdout prints `shared factory: interactive + worker rebuild OK (external System rejected)`.

## 3. Startup preflight — `preflight_boot` (standalone)

Register one `PreflightCheck`, run `PreflightRunner::run_all`, print statuses. Hosts
retain the returned results and auth-gate any setup UI.

```bash
cargo run -p higgs --example preflight_boot --features preflight
```

Success: stdout prints `preflight: demo-always-pass — Passed { … }`.

## 4. Axum session host — `axum_session_host` (standalone)

When to use: teach session middleware → Higgs context → shared worker factory on Axum
(mirrors `Higgs::from_request` assembly used inside Leptos server functions).

```bash
cargo run -p higgs --example axum_session_host --features ssr
```

Success: stdout prints `axum_session_host: OK — session → Higgs → worker factory`.

## 5. Leptos server function context — `server_fn_context` (standalone)

Real `provide_context(Arc<HiggsConfig>)` + Axum `Parts` (router / session extensions)
inside a Leptos `Owner`, then `#[higgs_macros::server(auth)]` → `Higgs::from_request`
→ `valence()`.

```bash
cargo run -p higgs --example server_fn_context --features ssr
```

Success: stdout prints `server_fn_context: OK — from_request + valence`.

## 6. Leptos server function backends — `server_fn_backends` (standalone)

Same Leptos Owner / `#[higgs_macros::server(auth)]` pattern as `server_fn_context`, with
Chronon / Boson / Photon registered on `HiggsConfig`. Proves SSR
`ctx.chronon()` / `ctx.boson()` / `ctx.photon()` (and `valence()`).

```bash
cargo run -p higgs --example server_fn_backends --features full,ssr
```

Success: stdout prints `server_fn_backends: OK — from_request + chronon/boson/photon accessors`.

## 7. Chronon job — `chronon_job` (standalone)

`#[chronon_coordinator_macros::script]` recovers Valence via `valence_from_context`. Boots
`HiggsConfig` with `.chronon(...)` (local in-memory backend) and invokes the registered
script through `ValenceScriptContextFactory`.

```bash
cargo run -p higgs --example chronon_job --features chronon
```

Success: stdout prints `chronon_job: OK — #[script] + valence_from_context + HiggsConfig.chronon`.

## 8. Boson task — `boson_task` (standalone)

`#[task]` recovers Valence via `valence_from_context`. Boots `HiggsConfig` with `.boson(...)`
(mem queue + `ValenceExecutionContextFactory`) and runs one job via `ManualWorker`.

```bash
cargo run -p higgs --example boson_task --features boson
```

Success: stdout prints `boson_task: OK — #[task] + valence_from_context + HiggsConfig.boson`.

## 9. Photon worker — `photon_worker` (standalone)

`#[topic]` / `#[subscribe]` rebuild Valence from the published event's actor JSON. Boots
`HiggsConfig` with `.photon(...)` and `ValenceIdentityFactory` on `start_executor`.

```bash
cargo run -p higgs --example photon_worker --features photon
```

Success: stdout prints `photon_worker: OK — #[subscribe] + Valence from event + HiggsConfig.photon`.

## Host composition (SSR + workers)

SSR examples show `Higgs::from_request` → Valence and optional subsystem accessors.
Worker examples show the family's macro handler recovering Valence from the identity
adapter (same factory Arc as `HiggsConfig`). Real deployments may split SSR and workers
into separate processes; hosts own layout, store URLs, and auth.

## Summary

| Example | Topology | Features | Notes |
|---------|----------|----------|-------|
| `config_boot` | Process-wide config | (none) | Mem Valence factory |
| `shared_factory` | Shared factory (same process) | `ssr` | Interactive + worker + actor policy |
| `preflight_boot` | Startup preflight | `preflight` | `PreflightRunner::run_all` |
| `axum_session_host` | Axum + session + worker | `ssr` | HTTP oneshot: deny anon / rebuild worker |
| `server_fn_context` | Leptos `from_request` | `ssr` | Owner + Parts + `#[server(auth)]` → valence |
| `server_fn_backends` | Leptos + backends | `full,ssr` | `from_request` → chronon/boson/photon |
| `chronon_job` | Chronon script macro | `chronon` | `#[script]` + `valence_from_context` |
| `boson_task` | Boson task macro | `boson` | `#[task]` + `valence_from_context` |
| `photon_worker` | Photon subscribe macro | `photon` | `#[subscribe]` + Valence from event |
