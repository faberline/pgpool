# Architecture

Pgpool is one root Rust package with one `pgpool` context.
A context is one product boundary with its own model and operations.
[ddd.toml](../ddd.toml) holds the machine-checked layout and policy.
[ADR 0001](adr/0001-standard-layout-and-ddd.md) records the choice.

## Context map

| Context | Form | Depends on | Owns |
|---|---|---|---|
| [pgpool](domain/pgpool.md) | Four layers in one package | No other local context | PostgreSQL admission, backend reuse, budgets, drain, operations, and provider seam. |

Shared Core libraries are external dependencies.
All Core git dependencies stay at tag `v0.4.14`.
Current Core documents provide layout terms only.
This layout does not adopt Core main P2 APIs.

## Layer map

Each layer may use itself.
The table lists other allowed local layers.

| Layer | May use | Current code |
|---|---|---|
| Domain | No other local layer | [Endpoint values](../src/domain/platform.rs), [budget and reserve values](../src/domain/k8s.rs), and [reactor state](../src/domain/pool/reactor_state.rs). |
| Application | Domain | [Runtime plan](../src/application/runtime_plan.rs), [pool and reactor](../src/application/pool.rs), [wire codec](../src/application/wire.rs), [proxy config](../src/application/proxy/config.rs), and [Kubernetes control](../src/application/k8s.rs). |
| Infrastructure | Domain | [Live PostgreSQL capacity and TLS discovery](../src/infrastructure/platform/discovery.rs). |
| Interfaces | Application | [Admin HTTP](../src/interfaces/admin.rs), [PostgreSQL proxy](../src/interfaces/proxy.rs), [pool entry](../src/interfaces/pool.rs), [operator](../src/interfaces/operator.rs), and [offline spec](../src/interfaces/spec.rs). |

Domain code uses pure values and deterministic state changes.
It does not open connections, read the environment, or read the current clock.
Application code drives current operations.
Infrastructure connects to live technology.
Interfaces accept input and produce protocol output.

The operator has two direct layer links listed below.
Their exact path and reasons are fixed in `ddd.toml`.

## App and API assembly

[src/app.rs](../src/app.rs) declares `cli`, `commands`, and `serve`.
`pgpool::app::run` is the process assembly entry.
The [binary main](../src/bin/pgpool/main.rs) calls it.
CLI parsing and command routing live under `src/app/`.

[src/lib.rs](../src/lib.rs) holds module declarations and public re-exports.
[src/api.rs](../src/api.rs) declares the public facade files.
A facade re-exports items under a stable public name.
Those files hold re-exports only.
App and API assembly appear under `[assembly]` in `ddd.toml`.
They sit outside the four layers.

Public module paths stay available:

```rust
pgpool::{pool, proxy, wire, admin, k8s, operator, platform, spec}
```

Root runtime names also stay available:

```rust
pgpool::{PoolMode, RuntimePlan, default_runtime_plan, runtime_plan_json}
```

`pgpool::proxy` combines application configuration with the interface edge.
`pgpool::k8s` combines domain budgets with application control and rendering types.
`pgpool::platform` combines domain values with the live discovery function.
A re-export does not change the defining layer of an item.
Implementation code uses internal layer paths.

## File layout

Root `Cargo.toml` defines the library and single binary.
There is no explicit one-member workspace or `crates/` tree.
The existing `--workspace` gates still select the root package.

Modules use named parent files.
For example, `src/application/pool.rs` declares children under `src/application/pool/`.
The Clippy `mod_module_files = "deny"` lint rejects `mod.rs`.
No `#[path]` module is needed.

The former 1,761-line transaction runtime is split into 12 files.
`src/application/pool/reactor/runtime.rs` is the type and module facade.
Its children are `types`, `loop`, `output`, `client`, `backend`, `backend_startup`, `backend_active`, `startup`, `close`, `socket`, and `tests`.
The largest current file in this group has 294 lines.
The split preserves actions, frame order, timeouts, and ownership transitions.

## Exact layer exceptions

B3 checks local layer dependency direction.
There are exactly two B3 exceptions.
Both apply only to [src/interfaces/operator/reconcile.rs](../src/interfaces/operator/reconcile.rs).

| Subject | Existing call | Reason |
|---|---|---|
| `interfaces->infrastructure` | Live PostgreSQL discovery | Reconcile calls the live adapter directly. Preserving its call and error behavior avoids an operator redesign in this layout change. |
| `interfaces->domain` | Endpoint value construction and classification | Reconcile constructs and classifies requests and facts with pure endpoint values. An application wrapper would redesign that call shape. |

The operator still has these two direct layer links.
No other exception or size suppression is part of this choice.

## File size debt

C1 warns when a measured source file exceeds 400 lines.
C2 errors when a measured source file exceeds 1,000 lines.
These nine C1 files remain existing debt.
C1 warnings stay visible in the report.
They are not `ddd.toml` exceptions or suppressions.

| Source | Lines |
|---|---:|
| [src/application/k8s/control.rs](../src/application/k8s/control.rs) | 785 |
| [src/application/pool/backend_pool.rs](../src/application/pool/backend_pool.rs) | 809 |
| [src/application/pool/reserve.rs](../src/application/pool/reserve.rs) | 473 |
| [src/application/wire/backend.rs](../src/application/wire/backend.rs) | 696 |
| [src/application/wire/frontend.rs](../src/application/wire/frontend.rs) | 694 |
| [src/application/wire/reader.rs](../src/application/wire/reader.rs) | 456 |
| [src/interfaces/operator/reconcile.rs](../src/interfaces/operator/reconcile.rs) | 480 |
| [src/interfaces/pool/transaction.rs](../src/interfaces/pool/transaction.rs) | 653 |
| [src/interfaces/proxy/relay.rs](../src/interfaces/proxy/relay.rs) | 448 |

## Tests and evidence

Regular integration modules live in `tests/it/`.
[tests/it/main.rs](../tests/it/main.rs) registers admin_plane, cli_contract, connection_discovery, operator, pgbouncer_benchmark, pool, pool_modes, proxy, reconcile_planning, session_proxy, and wire_codec.

[trust_startup_replay](../tests/trust_startup_replay.rs) and [transaction_extended_protocol](../tests/transaction_extended_protocol.rs) stay separate Cargo targets.
Both change `PGPOOL_TRANSACTION_ENGINE`.
Each starts with `//! isolation: changes PGPOOL_TRANSACTION_ENGINE`.
Separate executables keep their environment state out of the regular integration process.

The complete all-features layout baseline has 176 tests and zero benchmarks.
Acceptance compares the full before and after lists.
The count alone is not proof.
See [architecture conformance](operations/architecture-conformance.md) for the commands.

Current Rust files define behavior.
Historical generator records remain unchanged.
Local checks and byte comparisons do not establish a release or production acceptance.
