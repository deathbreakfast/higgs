# higgs-host

Host request extraction for server functions — database router, optional session
snapshot, and Valence actor derivation via the session identity contract.

```toml
higgs-host = { git = "https://github.com/unified-field-dev/higgs", branch = "main", package = "higgs-host", default-features = false, features = ["ssr"] }
```

Enable `ssr` for Axum/Leptos helpers:

- **`host_ctx`** / **`HostRequestCtx`** — router + session snapshot → `actor()` /
  `session_user_id()`
- **`unsafe_data_plane`** / **`DataPlaneCtx`** — router-only data plane
- **`require_session`** — fail closed for anonymous requests
- **`current_operation`** / **`with_operation`** — task-local operation name for
  attribution (used by `higgs-macros` and `Higgs::unsafe_system_valence`)

Session adapters implement `higgs_identity::SessionIdentity` and populate
`SessionSnapshot` in Axum extensions. `higgs` builds on these extractors via
`Higgs::from_request`.

Runnable teaching demos (including `Higgs::from_parts` + shared factory) live on the
`higgs` package — see [higgs/examples](../higgs/examples/README.md):

```bash
cargo run -p higgs --example shared_factory --features ssr
```

See the [workspace README](../README.md).
