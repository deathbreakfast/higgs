# higgs

Platform composition crate: process-wide `HiggsConfig` (Valence factory + optional
Chronon / Boson / Photon backends) and the SSR `Higgs::from_request` public crate. Hosts and
apps should depend on this package as **`higgs`** so macros resolve
`higgs::Higgs::from_request()` and `higgs::require_session()`.

```toml
higgs = { git = "https://github.com/unified-field-dev/higgs", branch = "main", default-features = false, features = ["ssr", "full"] }
```

Enable `ssr` for `Higgs::from_request`. Enable `chronon` / `boson` / `photon` (or `full`)
for subsystem accessors. Prefer `valence()` over `unsafe_system_valence()`. Use
`#[higgs_macros::server(auth)]` when a session is required. Install
`higgs::actor_policy::external_actor_json_policy()` on factories that rebuild from
external actor JSON.

## Runnable examples

Canonical path and host-composition notes:
[higgs/examples](examples/README.md).

```bash
cargo run -p higgs --example config_boot
cargo run -p higgs --example shared_factory --features ssr
cargo run -p higgs --example preflight_boot --features preflight
cargo run -p higgs --example server_fn_context --features ssr
cargo run -p higgs --example server_fn_backends --features full,ssr
cargo run -p higgs --example chronon_job --features chronon
cargo run -p higgs --example boson_task --features boson
cargo run -p higgs --example photon_worker --features photon
```

See the [workspace README](../README.md) and [SECURITY.md](../SECURITY.md) for the hero
example, boot wiring, and threat model.
