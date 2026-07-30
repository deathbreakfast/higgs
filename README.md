# Higgs

[![CI](https://github.com/unified-field-dev/higgs/actions/workflows/ci.yml/badge.svg)](https://github.com/unified-field-dev/higgs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[GitHub](https://github.com/unified-field-dev/higgs) · `cargo doc -p higgs --open`

Unified application context for hosts — one Valence factory and subsystem hub shared by
server functions, Chronon jobs, Boson tasks, and Photon handlers.

```rust
use higgs::Higgs;
use leptos::prelude::*;

#[higgs_macros::server(auth)]
pub async fn create_order(/* … */) -> Result<(), ServerFnError> {
    let ctx = Higgs::from_request().await?;

    // Request-scoped Valence with the current session actor.
    let valence = ctx.valence()?;

    // Prefer valence() for user work. System elevation is an explicit escape hatch:
    // let _sys = ctx.unsafe_system_valence()?;

    // With features = ["full"] (or chronon / boson / photon individually):
    // let _photon = ctx.photon()?;
    // let _chronon = ctx.chronon()?;
    // let _boson = ctx.boson()?;

    let _ = valence;
    Ok(())
}
```

## About

Higgs is a host wiring library that creates a process-wide shared context so interactive
requests and background work share the same identity and data path. A process-wide
`HiggsConfig` holds the Valence factory and optional Chronon / Boson / Photon backends.

- **Shared Valence factory** — `HiggsValenceFactory` builds actor-scoped `Valence` from
  serialized actor JSON. Hosts reuse it for Leptos server functions, Chronon
  `ContextFactory` runs, Boson `ExecutionContextFactory` tasks, and Photon handlers.
- **Request context** — `Higgs::from_request` (feature `ssr`) assembles router, session
  actor, and `HiggsConfig` for server functions — one call instead of hand-threaded
  globals.
- **Valence accessors** — `valence()` for the session actor; `unsafe_system_valence()`
  for intentional System work (elevates privacy — authorize first).
- **Operation attribution** — `#[higgs_macros::server]` wraps Leptos `#[server]` and
  tags the call with the function name; `#[higgs_macros::server(auth)]` also requires
  a session.
- **Subsystem accessors** — feature-gated `chronon()`, `boson()`, and `photon()` on
  `Higgs` / `HiggsConfig` when the host registers those backends at boot.
- **Startup preflight** — structured validation and seeding hooks after the database
  router is installed (`preflight` feature, enabled with `ssr`).
- **Identity boundary** — `higgs-identity` session snapshots and `higgs-host` extractors keep
  concrete user models out of the host request path; workers rebuild Valence from the
  same factory and captured actor JSON.

Mount Higgs in the host binary (enable `ssr` / `full`).

## Security notes

See [SECURITY.md](SECURITY.md) for the integrator threat model. Short version:

| Prefer | Avoid for user paths |
|--------|----------------------|
| `ctx.valence()` | `ctx.unsafe_system_valence()` without authz |
| `#[higgs_macros::server(auth)]` when signed-in | Raw `unsafe_data_plane` / `unsafe_database_router` for CRUD |
| `actor_policy::external_actor_json_policy()` on external factories | Client-supplied `Actor::System` JSON |

## Quick start

```toml
[dependencies]
higgs = { git = "https://github.com/unified-field-dev/higgs", branch = "main", default-features = false, features = ["ssr", "full"] }
higgs-macros = { git = "https://github.com/unified-field-dev/higgs", branch = "main", package = "higgs-macros" }
higgs-host = { git = "https://github.com/unified-field-dev/higgs", branch = "main", package = "higgs-host", default-features = false, features = ["ssr"] }
higgs-identity = { git = "https://github.com/unified-field-dev/higgs", branch = "main", package = "higgs-identity" }
```

Default features are empty — enable `ssr` for request context, and `chronon` / `boson` /
`photon` (or `full`) for subsystem accessors.

```rust
use std::sync::Arc;
use higgs::HiggsConfig;

let config = Arc::new(
    HiggsConfig::builder()
        .valence_factory(/* your HiggsValenceFactory */)
        // .chronon(scheduler, backend, registry)  // feature = "chronon"
        // .photon(photon)                         // feature = "photon"
        // .boson(backend)                         // feature = "boson"
        .build()?
);

// Provide Arc<HiggsConfig> in Leptos context; pass valence_factory() into workers.
```

Session middleware populates `higgs_identity::SessionSnapshot` in Axum extensions;
`Higgs::from_request` reads it to derive the Valence actor.

Runnable examples: [higgs/examples](higgs/examples/README.md).

## Verify

CI gates: [`docs/VERIFICATION.md`](docs/VERIFICATION.md).

```bash
export CARGO_BUILD_JOBS=1
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## Workspace

| Crate | Role |
|-------|------|
| `higgs-core` | Valence / session hub (config, preflight, request context) |
| `higgs` | Platform composition: Chronon / Boson / Photon accessors above `higgs-core` |
| `higgs-macros` | `#[server]` / `#[server(auth)]` with operation attribution |
| `higgs-host` | Host request extraction (`HostRequestCtx`, `unsafe_data_plane`) |
| `higgs-identity` | Session snapshot contract for hosts |

## License

MIT. See [LICENSE](LICENSE), [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md),
and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
