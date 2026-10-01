# Contributing to pgpool

## Brief

This page explains how to change `pgpool`.
[README.md](README.md) defines the product contract.
[STATUS.md](STATUS.md) states current support.
[ROADMAP.md](ROADMAP.md) keeps open outcomes.
The [workspace CONTRIBUTING.md](https://github.com/faberline/workspace/blob/main/CONTRIBUTING.md) defines shared authoring rules.

Use `product-deliver` for authorized work.
QA owns the red e2e case and its registration.
Dev owns the red unit case and scoped implementation.
PM owns authorized product documents.
A fresh `pgpool-qa` runs the declared complete gate.
The controller owns Git, tracker changes, and acceptance.
Legacy AW use is explicit-only.

## Source layout

Use the root single Rust package.
Keep pure values and state in `src/domain/`.
Keep pool, wire, runtime-plan, and control operations in `src/application/`.
Keep live PostgreSQL discovery in `src/infrastructure/`.
Keep HTTP, PostgreSQL frontend, spec, and operator edges in `src/interfaces/`.
Use `src/app/` to assemble the process.
Keep `src/api/` as public re-exports.

Use named parent files, such as `src/application/pool.rs`.
Do not add `mod.rs` or `#[path]` modules.
Keep `src/lib.rs` and `src/bin/pgpool/main.rs` thin.
Preserve `pgpool::{pool,proxy,wire,admin,k8s,operator,platform,spec}`.
Preserve `PoolMode`, `RuntimePlan`, `default_runtime_plan`, and `runtime_plan_json` at the crate root.
Keep all Core git dependencies at tag `v0.4.14`.

Current Rust files define the implementation.
Preserve historical `tech-design/` files and old generator markers.
Do not regenerate source from those records.
[Architecture](docs/architecture.md) records the two layer exceptions and nine file size warnings.

## Test layout

Regular integration cases live in `tests/it/`.
`tests/it/main.rs` registers 11 modules in one Cargo target named `it`.
Colocated unit cases stay with their source modules.

`tests/trust_startup_replay.rs` and `tests/transaction_extended_protocol.rs` remain separate Cargo targets.
Both change `PGPOOL_TRANSACTION_ENGINE`.
That environment value is shared by one process.
Their leading isolation notes explain why each needs its own executable.

Some cases need a reachable local PostgreSQL backend.
Some discovery cases need a TLS-only PostgreSQL fixture.
Read skip messages before treating a pass as external evidence.

## Verification

The document checker requires Python 3.11 or later.

Run the commands from the root of the candidate package.
The `--workspace` flag remains valid for this root single package.
The complete local gate includes all four feature sets below.

```sh
git -c core.fsmonitor=false diff --check
cargo fmt --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo check --locked --workspace --all-targets --features self-update
cargo clippy --locked --workspace --all-targets --features self-update -- -D warnings
cargo test --locked --workspace --features self-update
cargo check --locked --workspace --all-targets --features issue
cargo clippy --locked --workspace --all-targets --features issue -- -D warnings
cargo test --locked --workspace --features issue
cargo check --locked --workspace --all-targets --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-targets --all-features
cargo test --locked --workspace --all-features --test it
cargo test --locked --workspace --all-features --test trust_startup_replay
cargo test --locked --workspace --all-features --test transaction_extended_protocol
/opt/homebrew/bin/python3.11 /Users/chrischeng/faberlines/workspace/scripts/meta/project_docs_contract.py check .
uv run --isolated --no-project /Users/chrischeng/faberlines/workspace/scripts/meta/rust_arch_contract.py check --root /Users/chrischeng/faberlines --repo . --strict --base main --json /private/tmp/pgpool-rust-architecture.json
```

For an existing TLS-only fixture, set `PGPOOL_TLS_DISCOVERY_PORT` and `PGPOOL_TLS_DISCOVERY_CA` to its real port and CA file.
Then run the exact case with its module prefix.

```sh
cargo test --locked --workspace --all-features --test it connection_discovery::cloudsql_discovery_succeeds_against_tls_required_postgres -- --exact
```

The fixture helper is `tests/tls_required_discovery.sh`.
Its selector must use target `it` and the full case name above.
A local pass does not prove a Cloud SQL deployment or full relay TLS support.
The [conformance guide](docs/operations/architecture-conformance.md) gives byte comparisons and evidence limits.
