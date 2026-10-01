# Pgpool domain

## Purpose

Pgpool owns PostgreSQL connection pooling for application clients.
It owns frontend admission, backend reuse, budgets, drain, pool health, and admin routes.
PostgreSQL owns durable database data.
The [capability contract](../../README.md#capabilities) retains the 15 product names.

## Model and operations

| Concern | Source | Responsibility |
|---|---|---|
| Endpoint and capacity facts | [domain/platform.rs](../../src/domain/platform.rs) | Provider, role, endpoint, advisory limit, and connection facts. |
| Endpoint budgets | [domain/k8s/budget.rs](../../src/domain/k8s/budget.rs) | Deterministic capacity and quota decisions. |
| Reserve values | [domain/k8s/reserve.rs](../../src/domain/k8s/reserve.rs) | Pure grant and demand values. |
| Reactor state | [domain/pool/reactor_state.rs](../../src/domain/pool/reactor_state.rs) | Ownership, waiters, and deterministic actions. |
| Backend reuse | [application/pool](../../src/application/pool.rs) | Open, lease, reset, reuse, and close under a bound. |
| Wire handling | [application/wire](../../src/application/wire.rs) | Decode, validate, and encode PostgreSQL frames. |
| Quota and drain control | [application/k8s/control.rs](../../src/application/k8s/control.rs) | Admit quotas and release them after drain. |
| Live discovery | [infrastructure/platform/discovery.rs](../../src/infrastructure/platform/discovery.rs) | Query limits with the configured transport and trust material. |
| Client and admin edges | [interfaces/pool](../../src/interfaces/pool.rs), [interfaces/proxy](../../src/interfaces/proxy.rs), and [interfaces/admin](../../src/interfaces/admin.rs) | Accept clients, relay protocol bytes, and expose routes. |
| Kubernetes edge | [interfaces/operator](../../src/interfaces/operator.rs) | Reconcile CRD, workload, quota, and status contracts. |

## Ownership rules

A session owner keeps its backend until the session ends.
A transaction owner keeps its backend until completion and reset permit reuse.
Backend capacity stays bounded while owners wait.
Dropping a lease returns capacity through the cleanup path.
Drain stops new admission and waits within the configured timeout.
The layout move keeps these rules.

The pooler uses a stateless Kubernetes Deployment.
PostgreSQL keeps durable data.
The operator admits endpoint quotas before apply.
It keeps the drain-before-release order for quota changes.

## Provider boundary

Cloud SQL, AlloyDB, and plain PostgreSQL remain explicit provider choices.
Adapters supply endpoints and authentication material.
The core runtime does not embed a provider SDK.
Discovery collects runtime connection facts.
Pure values combine those facts with advisory limits.

Operator reconcile still calls discovery directly.
It also constructs and classifies endpoint values directly.
These are the two exact B3 exceptions in [ddd.toml](../../ddd.toml).
[Architecture](../architecture.md#exact-layer-exceptions) gives their reasons.

## Public language

The application layer owns pool, wire, proxy config, runtime-plan, and control operations.
`src/api/` re-exports them under the existing public paths.
`pgpool::{pool,proxy,wire,admin,k8s,operator,platform,spec}` stays available.
`PoolMode`, `RuntimePlan`, `default_runtime_plan`, and `runtime_plan_json` stay at the crate root.
`pgpool::app::run` assembles the process.

## Open product boundaries

Transaction mode still rejects the extended query protocol.
Full competitor parity and the isolated performance ratchet remain open.
The full external EC suite and security evidence remain open.
A true long-run soak and restart or rolling deployment acceptance remain open.
Provider authentication and full adapter conformance remain separate work.
Images and repeatable Kubernetes release acceptance remain open.

[STATUS.md](../../STATUS.md) states the local supported slice.
[ROADMAP.md](../../ROADMAP.md) names completion evidence.
Historical `tech-design` and generator markers remain evidence records.
Current Rust files define behavior.
