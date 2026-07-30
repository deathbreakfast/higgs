# higgs verification

Unified application context (Valence factory + optional Chronon / Boson / Photon accessors).
Re-run after code or doc changes. Covered by unit + integration tests below.

## Environment

```bash
export CARGO_BUILD_JOBS=1
```

## Unit + integration (CI)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
```

Narrower runs:

```bash
cargo test -p higgs
cargo test -p higgs-core --all-features
cargo test -p higgs-identity
cargo test -p higgs-host --features ssr
cargo test -p higgs-core --features ssr,preflight --test session_server_contract --test preflight_contract
cargo test -p higgs --features test-utils,full --test config_contract
cargo test -p higgs-host --features ssr --test host_session_contract
cargo test -p higgs-identity --test session_contract
```

### TEST_MAP

| Behavior | Level | Happy | Sad | Notes |
|----------|-------|-------|-----|-------|
| `HiggsValenceFactory` object-safe + builder | unit | factory set / Arc factory | missing factory → err | `higgs-core::tests` |
| `external_actor_json_policy` | unit | User actor allowed | System rejected for External | `actor_policy::tests` |
| `HiggsError::internal` display | unit | opaque `"internal higgs error"` | — | no secret leakage |
| `permission_*_payload` / parse | unit | denied / check-failed round-trip | unrelated / empty prefix → `None` | `server_runtime::tests` |
| `with_operation` task-local | unit | sets + clears operation | — | `server_runtime` + `higgs-host` |
| `Higgs::unsafe_system_valence` + operation | unit | System actor carries op name | capture factory → `Internal` | `context::tests` |
| `unsafe_database_router` | unit | matches host router Arc | — | `context::tests` |
| `SessionSnapshot` / `auth_hash_eq` | unit | new / identity / eq | mismatch / short hash | `higgs-identity::tests` |
| `HostRequestCtx` actor mapping | unit | session → User; missing → Anonymous | — | `higgs-host::ssr::tests` |
| `SessionSnapshot` / `SessionIdentity` | integ | to_snapshot; serde round-trip | inequality on id/hash mismatch | `session_contract` |
| `HostRequestCtx` / nested `with_operation` | integ | session → User; nested TLS scopes | — | `host_session_contract` |
| `Higgs::from_parts` / `valence` / permissions | integ | session → usable Valence; payload round-trip | factory fail → opaque `Internal`; garbage parse → `None` | `session_server_contract` |
| `PreflightRunner` / store snapshot | integ | empty / pass / store | failed check status preserved | `preflight_contract` |
| Platform config + subsystem accessors | integ | builder / `From` core | missing factory; unset chronon/boson/photon | `config_contract` |

## Layer 2 — E2E

**Waived.** Host session and server-runtime contracts (identity snapshot → actor →
Valence; permission payload encode/decode; preflight runner) are exercised by Layer 1
integration tests named below. A gluon-scale `*-e2e` / IsolatedLab crate is not
warranted for this library hub; live Axum + Leptos request extraction belongs in host
product e2e if needed.

Covering integ tests:

- `session_identity_to_snapshot_happy_path` / `session_snapshot_serde_roundtrip_happy_path`
- `authenticated_session_maps_to_user_actor_happy_path` / `with_operation_nested_scopes_happy_path`
- `higgs_from_parts_session_valence_happy_path` / `higgs_valence_factory_failure_sad`
- `system_valence_with_operation_happy_path`
- `server_runtime_permission_denied_roundtrip_happy_path` / `server_runtime_parse_rejects_garbage_sad`
- `preflight_runner_pass_happy_path` / `preflight_store_snapshot_happy_path` / `preflight_runner_failed_check_sad`
- `platform_config_builder_happy_path` / `chronon_accessors_without_config_sad`

## Notes

- Tests may `unwrap`/`expect`; production paths map failures to [`HiggsError`](../higgs-core) /
  `anyhow` (no ordinary-path unwrap).
- Sad-path assertions check typed variants or opaque display strings — (stronger than `is_err()` alone).
- Happy-path tests are named `*_happy_path` so audits detect them.
- Runnable teaching hosts: [`higgs/examples/README.md`](../higgs/examples/README.md).
