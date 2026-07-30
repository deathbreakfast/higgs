# higgs-identity

Abstract auth/session identity surface for host crates — `SessionSnapshot`,
`SessionIdentity`, and `SessionUserId`.

```toml
higgs-identity = { git = "https://github.com/unified-field-dev/higgs", branch = "main", package = "higgs-identity" }
```

Host middleware populates `SessionSnapshot` in Axum extensions. `higgs-host` and `higgs`
read it to build Valence actors without importing concrete user schemas. Product
identity models implement `SessionIdentity` in adapter crates (e.g. `lepton-host-adapter`).

See the [workspace README](../README.md).
