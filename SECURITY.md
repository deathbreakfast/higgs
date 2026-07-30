# Security Policy

## Supported versions

Security fixes are accepted against the latest `main` branch and tagged releases (`0.1.x` / `0.2.x`) of this repository's crates (`higgs`, `higgs-macros`, `higgs-host`, `higgs-identity`).

## Reporting a vulnerability

Please **do not** open a public GitHub issue for security-sensitive reports.

Prefer one of the following:

1. **GitHub Security Advisories** — use [Report a vulnerability](https://github.com/unified-field-dev/higgs/security/advisories/new) on this repository when available.
2. Contact the maintainers privately via the repository owner listed at https://github.com/unified-field-dev/higgs.

Include:

- a description of the issue and its impact
- steps to reproduce or a proof of concept when possible
- affected crate names and versions

We will acknowledge receipt as soon as practical and coordinate a fix and disclosure timeline with you.

## Scope

In scope: vulnerabilities in this repository's published crates and documentation that could cause unsafe production defaults, plus CI/supply-chain issues in this repository.

Out of scope: vulnerabilities solely in third-party dependencies unless this project mishandles them in a security-relevant way.

## Integrator threat model (SSR vs workers)

Higgs is a host-wiring library: it does not implement cookies, CORS, or HTTP auth. Security depends on host middleware and how apps call privileged APIs.

| Surface | Safe default | Escape hatch |
|---------|--------------|--------------|
| User data access | `Higgs::valence()` (session / Anonymous actor) | — |
| System-elevated Valence | **Avoid** for user-driven work | `Higgs::unsafe_system_valence()` only after host-local authz |
| Raw DB router | Avoid for user CRUD | `unsafe_database_router` / `higgs_host::unsafe_data_plane` |
| Server functions | `#[higgs_macros::server]` (public OK) | `#[higgs_macros::server(auth)]` requires session |
| Preflight results | Keep `PreflightRunner::run_all` return value in host state; auth-gate setup UI | No process-global snapshot API |
| External actor JSON | Install `higgs::actor_policy::RejectExternalSystemActor` on factories that rebuild from untrusted JSON | SSR `unsafe_system_valence` is an explicit System mint (not an external JSON path) |

### Why not `unsafe_system_valence`

`unsafe_system_valence` builds a Valence with `Actor::System`. That actor bypasses entity and field privacy policies. Using it for ordinary request CRUD is a privacy hole: every row the schema would have denied becomes readable or writable.

Hosts should:

1. Gate the server function with `require_session` / `#[higgs_macros::server(auth)]` (and app permission checks when needed).
2. Call `Higgs::valence()` so the session (or Anonymous) actor is subject to Valence policies.
3. If a query fails under session Valence, fix the schema policy or the auth gate — do not elevate.

There is no soft-named `system_valence` alias; only the explicit `unsafe_*` name remains.

### Host checklist

1. Session middleware is the only writer of `higgs_identity::SessionSnapshot`.
2. Prefer `valence()`; treat `unsafe_*` APIs as control-plane only.
3. Authorize in the server function before enqueue/publish/schedule; set actor JSON server-side (never accept client `Actor::System`).
4. Wire `RejectExternalSystemActor` on factories that rebuild from external actor JSON (`higgs::actor_policy::external_actor_json_policy()`). Use `ActorTrust::Internal` only for intentional SSR elevation paths.
5. Mount coordinator admin HTTP with authentication enabled; do not leave transport or admin bypass env flags set in production.
6. Do not put secrets in preflight messages or client-facing `HiggsError` strings (`Internal` is opaque by design).
