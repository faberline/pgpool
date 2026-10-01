# Architecture conformance

## Purpose

Check the standard layout while preserving the public contract.
Run checks from the candidate package root.
Use current source and retained before captures.
The controller owns final acceptance.

## Document check

The checker requires Python 3.11 or later.

The checker finds its Git checkout from the current directory.
Run the workspace script by absolute path from the candidate package.
A workspace current directory with an outside project path is not supported.

```sh
/opt/homebrew/bin/python3.11 /Users/chrischeng/faberlines/workspace/scripts/meta/project_docs_contract.py check .
```

The checker validates README, STATUS, and ROADMAP structure.
It resolves source labels, links, and gate names.
It does not run product gates or decide production readiness.

## Source and feature gates

Run the complete set in [CONTRIBUTING.md](../../CONTRIBUTING.md#verification).
It covers default, `self-update`, `issue`, and all features.
Each set has check, Clippy, and test commands.
The existing `--workspace` flag still selects the root package.

Run the strict architecture check:

```sh
uv run --isolated --no-project /Users/chrischeng/faberlines/workspace/scripts/meta/rust_arch_contract.py check --root /Users/chrischeng/faberlines --repo . --strict --base main --json /private/tmp/pgpool-rust-architecture.json
```

The required result has no active errors.
Only the nine C1 warnings in [Architecture](../architecture.md#file-size-debt) are expected.
Only the two exact B3 operator reconcile exceptions in `ddd.toml` are allowed.
Do not add a suppression to make the check pass.

## Test targets and isolation

Run the regular target and both isolated targets:

```sh
cargo test --locked --workspace --all-features --test it
cargo test --locked --workspace --all-features --test trust_startup_replay
cargo test --locked --workspace --all-features --test transaction_extended_protocol
```

`tests/it/main.rs` registers 11 regular modules.
The other targets change `PGPOOL_TRANSACTION_ENGINE`.
Their leading notes explain isolation.
Keep them out of `tests/it/main.rs`.

Some cases return early when PostgreSQL is absent.
The TLS case also returns early when fixture variables are absent.
Read skip messages and fixture state.
A zero exit alone does not prove an external case ran.

The fixture helper is `tests/tls_required_discovery.sh`.
Its selector must name `it` and the full case below.
For a running fixture, set `PGPOOL_TLS_DISCOVERY_PORT` and `PGPOOL_TLS_DISCOVERY_CA` to the real values.

```sh
cargo test --locked --workspace --all-features --test it connection_discovery::cloudsql_discovery_succeeds_against_tls_required_postgres -- --exact
```

This case checks configured TLS for live discovery.
It does not prove frontend or backend relay TLS.
This page does not claim that external TLS smoke ran.

## Layout parity

Use the clean original `main` HEAD commit as the baseline for this layout-only landing.
The candidate contains only Rust layout changes.
Keep all original Cloud SQL work in progress in the persistent working directory.
Leave that work uncommitted and separate from this layout-only commit.

Capture the complete test list and build the same binary feature set:

```sh
cargo test --locked --workspace --all-features -- --list
cargo build --locked --workspace --all-features --bin pgpool
```

The known baseline has 176 tests and zero benchmarks.
Compare the full inventories with the approved target and case mapping.
The eleven ordinary Cargo targets become the `it` target.
Their case names use the approved module prefixes.
Compare leaf-name multisets for both full inventories.
A leaf name is the final part of a case name.
A multiset records each name and its occurrence count.
Require evidence that each case body and its test attributes are preserved.
Counts alone are not proof.

Capture each binary surface before and after:

```sh
target/debug/pgpool --help
target/debug/pgpool runtime-plan
target/debug/pgpool spec --format openapi
target/debug/pgpool spec --format openapi-yaml
target/debug/pgpool spec --format json-schema
target/debug/pgpool spec --format routes
target/debug/pgpool k8s crd render
target/debug/pgpool k8s operator render
target/debug/pgpool k8s instance render --profile dev
target/debug/pgpool k8s instance render --profile staging
target/debug/pgpool k8s instance render --profile prod
target/debug/pgpool k8s instance render --profile template
```

Use `cmp` for every before and after CLI and artifact capture.
Require raw byte equality for those captures.
Preserve spec, CRD, and manifest bytes.
Preserve help and operational markers.
Preserve all public module and root runtime names.
Keep `Cargo.lock` and Core tags at `v0.4.14`.

## Evidence limits

Document checks prove structure and resolvable gates.
Architecture checks prove configured layout rules.
Cargo gates prove the cases they actually run.
Byte comparisons prove the captured bytes.

These checks do not establish full parity or a current performance win.
They do not complete a soak, security review, or cluster release.
Historical work-item and evidence notes remain unverified in [STATUS.md](../../STATUS.md#evidence-policy).
