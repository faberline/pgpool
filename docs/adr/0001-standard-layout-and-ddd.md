# ADR 0001: Standard layout and DDD layers

## Context

Pgpool had one root package in an explicit one-member workspace.
Feature source lived under top-level feature modules.
The old transaction runtime held 1,761 lines.
Regular integration cases used separate Cargo targets.
Current Rust files define the implementation.
Historical `tech-design`, `SPEC-MANAGED`, and `HANDWRITE` records remain unchanged.

The authorized scope is a layout change.
It preserves product rules, public paths, CLI behavior, artifact bytes, and test cases.
It keeps Core git dependencies at tag `v0.4.14`.
It does not adopt Core main P2 APIs.

## Decision

Keep one root Rust package.
Remove the explicit one-member workspace.
Declare one `pgpool` context in [ddd.toml](../../ddd.toml).
Use four layers under `src/`.

| Layer | Owns |
|---|---|
| `domain` | Pure endpoint, quota, reserve, and reactor state values. |
| `application` | Runtime plan, pool and reactor operations, wire, proxy config, and Kubernetes control. |
| `infrastructure` | Live PostgreSQL capacity and TLS discovery. |
| `interfaces` | HTTP, PostgreSQL frontend, spec, and operator edges. |

Use `src/app/` for process assembly.
`pgpool::app::run` is its entry.
Binary main calls it.
Use `src/api/` for re-export facades.
Keep `src/lib.rs` thin.

Preserve the public module paths:

```rust
pgpool::{pool, proxy, wire, admin, k8s, operator, platform, spec}
```

Preserve crate-root runtime names:

```rust
pgpool::{PoolMode, RuntimePlan, default_runtime_plan, runtime_plan_json}
```

Use named parent files.
The Clippy `mod_module_files = "deny"` lint rejects `mod.rs`.
Use internal layer paths in implementation imports.
The checker resolves facades to the defining layer.

## Existing exceptions

Keep exactly two B3 exceptions.
Both apply only to `src/interfaces/operator/reconcile.rs`.

| Subject | Reason |
|---|---|
| `interfaces->infrastructure` | Existing reconcile calls live PostgreSQL discovery directly. Preserve the call and error behavior. An application port would redesign the operator. |
| `interfaces->domain` | Existing reconcile constructs and classifies discovery requests and facts with pure endpoint values. Preserve the call shape. An application wrapper would redesign it. |

The operator still has these direct layer links.
No other B3 exception is part of this choice.

## File size debt

Split the former transaction runtime into its facade and 11 child files.
The group has 12 files.
Its largest file has 294 lines.
Preserve actions, timeouts, frame order, and ownership transitions.

These nine existing C1 warnings remain visible.
The threshold is 400 lines.
They are not suppressions or layer exceptions.

| Source | Lines |
|---|---:|
| [src/application/k8s/control.rs](../../src/application/k8s/control.rs) | 785 |
| [src/application/pool/backend_pool.rs](../../src/application/pool/backend_pool.rs) | 809 |
| [src/application/pool/reserve.rs](../../src/application/pool/reserve.rs) | 473 |
| [src/application/wire/backend.rs](../../src/application/wire/backend.rs) | 696 |
| [src/application/wire/frontend.rs](../../src/application/wire/frontend.rs) | 694 |
| [src/application/wire/reader.rs](../../src/application/wire/reader.rs) | 456 |
| [src/interfaces/operator/reconcile.rs](../../src/interfaces/operator/reconcile.rs) | 480 |
| [src/interfaces/pool/transaction.rs](../../src/interfaces/pool/transaction.rs) | 653 |
| [src/interfaces/proxy/relay.rs](../../src/interfaces/proxy/relay.rs) | 448 |

C2 is the 1,000-line error threshold.
This choice adds no large-file suppression.

## Test decision

Put regular integration modules in `tests/it/`.
Register them in `tests/it/main.rs`.
Keep `tests/trust_startup_replay.rs` and `tests/transaction_extended_protocol.rs` as separate targets.
Both change `PGPOOL_TRANSACTION_ENGINE` and carry an isolation note.
Separate processes keep that environment state out of the regular target.

Keep every test case.
The complete baseline has 176 tests and zero benchmarks.
Acceptance compares the full lists and required artifact captures.
Counts alone do not prove preservation.

## Consequences

Directory names show each code role.
Public callers keep their module paths.
The process entry stays small.
The operator retains two direct layer links.
Nine size warnings remain debt.

The move keeps provider behavior, networking, manifests, benchmarks, and product rules.
In-progress Cloud SQL changes remain separate integration work at their moved paths.
The controller owns their preservation and final byte comparisons.

## Verification boundary

[Architecture conformance](../operations/architecture-conformance.md) gives the required checks.
They cover the feature matrix, isolated targets, architecture, documents, and bytes.
A required command is not a recorded pass.
Local checks do not establish a release, production deployment, or live tracker result.
