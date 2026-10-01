# Pgpool

## Brief

`pgpool` is the working name for Axiom's Kubernetes PostgreSQL connection pooler.
It owns frontend admission, backend budgets, pool reuse, drain, and the admin HTTP surface.
Cloud SQL, AlloyDB, and other provider connectors stay explicit adapters above the core runtime.

## Primary workflow

1. Build the local binary with `cargo build --locked --bin pgpool`.
2. Read `target/debug/pgpool runtime-plan` and `target/debug/pgpool spec --format routes`.
3. Run `target/debug/pgpool serve --backend-host 127.0.0.1 --backend-port 5432` for a local PostgreSQL backend.
4. Connect clients to the PostgreSQL frontend and inspect the admin routes.
5. Inspect `target/debug/pgpool k8s crd render`, `target/debug/pgpool k8s operator render`, and `target/debug/pgpool k8s instance render`.

The current CLI requires an explicit command.
`serve` starts the data and admin planes.
The old serve-by-default outcome remains in [ROADMAP.md](ROADMAP.md#default-serve-entrypoint).

## PostgreSQL pooling

The runtime supports bounded session and transaction pooling.
Session mode holds a backend for a client session.
Transaction mode returns a backend after the transaction and reset steps.
The transaction reactor uses one owner for socket state and buffers.
Transaction mode rejects the extended query protocol with the existing `0A000` error.

The pooler is stateless.
PostgreSQL owns durable database data.
The operator combines discovered and configured endpoint limits before quota admission.
The pooler drains before its endpoint quota is released.

Provider adapters supply endpoints and authentication material.
The runtime does not require a provider SDK.
Managed discovery TLS covers the discovery connection.
Full relay TLS and admin authorization evidence remain open.

## Source and package boundaries

This repository has one root Rust package.
It has a library and one `pgpool` binary.
It has no explicit Cargo workspace table or `crates/` tree.
The existing `--workspace` gate flag still selects this package.

[ddd.toml](ddd.toml) declares one `pgpool` context.
A context is one product boundary with its own model and operations.
Code lives in `domain`, `application`, `infrastructure`, and `interfaces` layers.
[src/app](src/app.rs) assembles the process.
[src/api](src/api.rs) preserves the public module names.

Core git dependencies remain pinned to `v0.4.14`.
This layout does not adopt Core main P2 APIs.
Current Rust files define the implementation.
Old `SPEC-MANAGED`, `HANDWRITE`, and [tech-design](tech-design) records remain historical.

## Contract discovery

| Fact | Current source | Discovery |
|---|---|---|
| CLI commands and flags | [app/cli.rs](src/app/cli.rs) | `pgpool --help` and `pgpool llm --topic workflow` |
| Runtime defaults | [application/runtime_plan.rs](src/application/runtime_plan.rs) | `pgpool runtime-plan` |
| Offline routes and schema | [interfaces/spec.rs](src/interfaces/spec.rs) | `pgpool spec --format routes` and `pgpool spec --format openapi` |
| Served admin contract | [interfaces/admin/router.rs](src/interfaces/admin/router.rs) | `/openapi.json` and `/docs` |
| Kubernetes assets | [interfaces/operator](src/interfaces/operator.rs) and [application/k8s](src/application/k8s.rs) | `pgpool k8s crd render`, `pgpool k8s operator render`, and `pgpool k8s instance render` |
| Current support | [STATUS.md](STATUS.md) | Support matrix |
| Required local checks | [CONTRIBUTING.md](CONTRIBUTING.md) | Full feature matrix and isolated targets |

## Capabilities

The 15 names below keep the existing product contract.
The listed gates check current local code.
A local pass does not complete an open external, production, or release boundary.
[STATUS.md](STATUS.md) states the supported scope.
[ROADMAP.md](ROADMAP.md) keeps the open outcomes.
The source label `external:pgpool` names this standalone repository.

### Capability index

| Capability | ID | User promise | Sources |
|---|---|---|---|
| Working-Name App Scaffold | `working-name-app-scaffold` | Keep one stable package and process name while the product name is open. | `external:pgpool` |
| Shared Server Substrate Adoption | `shared-server-substrate-adoption` | Use the shared libraries for TCP, HTTP, budgets, and drain. | `external:pgpool` |
| PostgreSQL Pooler Core | `postgresql-pooler-core` | Bound connections and reuse backends by pool mode. | `external:pgpool` |
| Platform Adapter Boundary | `platform-adapter-boundary` | Keep provider endpoints and authentication outside the pooler core. | `external:pgpool` |
| CLI Interface | `cli-interface` | Expose one binary with serve and offline plan, spec, and render commands. | `external:pgpool` |
| CLI Standard Surface | `cli-standard-surface` | Ship llm, upgrade, and issue commands with build data. | `external:pgpool` |
| Chainable Output Conformance | `chainable-output-conformance` | Keep raw artifacts usable and give operational output its next command. | `external:pgpool` |
| Competitor Feature Parity | `competitor-feature-parity` | Cover the functions needed for PgBouncer, Odyssey, and pgcat replacement. | `external:pgpool` |
| Competitor Performance | `competitor-performance` | Bind throughput and latency claims to a repeatable isolated comparison. | `external:pgpool` |
| EC Gates Configured | `ec-gates-configured` | Keep meter and guard evidence contracts explicit and runnable. | `external:pgpool` |
| HTTP/2 API List | `http2-api-list` | Publish the admin route list with matching served and offline specs. | `external:pgpool` |
| Kubernetes-Native Deployment | `kubernetes-native-deployment` | Render layered assets and keep quota and drain decisions explicit. | `external:pgpool` |
| Long-Running Stability | `long-running-stability` | Keep capacity and pool state safe through drain, restart, and long runs. | `external:pgpool` |
| Security Hardening | `security-hardening` | Protect auth, wire handling, TLS posture, and admin exposure. | `external:pgpool` |
| Standard Operational Endpoints | `standard-operational-endpoints` | Serve probes, metrics, live spec, and docs on one admin port. | `external:pgpool` |

### Working-Name App Scaffold

- ID: `working-name-app-scaffold`
- Promise: Keep `pgpool` as the package, binary, repository, and project name until the final name is set.
- Sources:
  - `external:pgpool` holds the root [package](Cargo.toml), the thin [binary entry](src/bin/pgpool/main.rs), and the project record in [aw.toml](aw.toml).
- Gate: `cargo test --locked -p pgpool --all-features`

The package is a single root Rust package. The offline runtime plan and route inventory remain available. The working name is not a final product naming decision.

### Shared Server Substrate Adoption

- ID: `shared-server-substrate-adoption`
- Promise: Use the shared TCP and HTTP server libraries. Use the shared lifecycle types for admission, readiness, and drain.
- Sources:
  - `external:pgpool` composes server-lifecycle, server-tcp, and server-http options in the [runtime plan](src/application/runtime_plan.rs) and connects them in [serve](src/app/serve.rs).
- Gate: `cargo test --locked -p pgpool --all-features`

The data and admin planes share their configured budget and drain state. The admin plane supports HTTP/1.1 and h2c. h2c means HTTP/2 without TLS. Core dependencies stay at `v0.4.14`.

### PostgreSQL Pooler Core

- ID: `postgresql-pooler-core`
- Promise: Provide PostgreSQL wire admission and bounded backend reuse. Support session and transaction pooling. Provide graceful drain and pool measurements.
- Sources:
  - `external:pgpool` implements reuse in [application/pool](src/application/pool.rs), wire frames in [application/wire](src/application/wire.rs), and the [pool edge](src/interfaces/pool.rs).
- Gate: `cargo test --locked -p pgpool --all-features`

The default frontend bind is `0.0.0.0:6432`. Session mode holds a backend for one client session. Transaction mode reuses a backend after the transaction and reset steps. Transaction mode rejects the extended query protocol with the existing `0A000` error. Full replacement breadth remains open.

### Platform Adapter Boundary

- ID: `platform-adapter-boundary`
- Promise: Keep Cloud SQL, AlloyDB, and plain PostgreSQL adapters explicit. Let adapters supply endpoints and authentication material through the existing seam. Keep provider SDKs outside the core runtime.
- Sources:
  - `external:pgpool` defines [endpoint and capacity values](src/domain/platform.rs), connects through [live discovery](src/infrastructure/platform/discovery.rs), and checks [discovery behavior](tests/it/connection_discovery.rs).
- Gate: `cargo test --locked -p pgpool --all-features`

Discovery uses provider and endpoint role values. It combines runtime limits with advisory limits. Managed-provider discovery can use configured TLS trust material. Provider authentication and full adapter conformance remain open. Discovery TLS covers the discovery connection.

### CLI Interface

- ID: `cli-interface`
- Promise: Provide one runnable `pgpool` binary. Keep the process entry and offline runtime-plan and spec commands stable.
- Sources:
  - `external:pgpool` declares [CLI commands](src/app/cli.rs), routes [commands](src/app/commands.rs), and checks the built binary in [CLI tests](tests/it/cli_contract.rs).
- Gate: `cargo test --locked -p pgpool --all-features`

Run `pgpool serve` to start the current data and admin planes. The parser requires a command. `pgpool k8s` supplies render and operator commands. The old serve-by-default outcome remains open.

### CLI Standard Surface

- ID: `cli-standard-surface`
- Promise: Ship the shared `llm`, `upgrade`, and `issue` surface. Include the build version, Git revision, build time, and target.
- Sources:
  - `external:pgpool` wires cli-std commands in [app/commands](src/app/commands.rs), uses build-stamp through [build.rs](build.rs), and checks [CLI help](tests/it/cli_contract.rs).
- Gate: `cargo test --locked -p pgpool --all-features`

`pgpool llm` supports outline, workflow, api, and boundaries topics. `pgpool issue search|view|create|comment` uses the pgpool project scope. Online commands need the matching `self-update` or `issue` feature and external service access. Local help checks do not prove a published release or a live tracker action.

### Chainable Output Conformance

- ID: `chainable-output-conformance`
- Promise: Keep spec and manifest output as raw artifact bytes. Give operational output the required next or terminal marker.
- Sources:
  - `external:pgpool` writes artifacts and the runtime-plan marker in [app/commands](src/app/commands.rs), with checks in [CLI tests](tests/it/cli_contract.rs).
- Gate: `cargo test --locked -p pgpool --all-features`

`runtime-plan` prints its plan followed by `next: pgpool spec --format routes`. Spec formats are `openapi`, `openapi-yaml`, `json-schema`, and `routes`. The next marker is separate from the plan JSON. Raw spec output has no operational wrapper.

### Competitor Feature Parity

- ID: `competitor-feature-parity`
- Promise: Cover transaction and session pooling, bounded admission, drain, and pool measurements for Axiom workloads that use PgBouncer, Odyssey, or pgcat.
- Sources:
  - `external:pgpool` holds the current [pool mode cases](tests/it/pool_modes.rs) and [admin cases](tests/it/admin_plane.rs).
- Gate: `cargo test --locked -p pgpool --all-features`

The local gate checks the existing pooler slice. It covers reuse, reset, admission limits, and admin behavior. The full external parity matrix remains open. The local gate does not prove complete replacement of a named competitor.

### Competitor Performance

- ID: `competitor-performance`
- Promise: Measure pooled connection throughput and latency with a repeatable vat-isolated meter gate. Keep external pooler comparisons advisory until the enforced gate exists.
- Sources:
  - `external:pgpool` holds the fixed comparison [runner](benchmarks/pgbouncer-transaction-pooling/run.sh), its [contract](benchmarks/pgbouncer-transaction-pooling/README.md), and [profile and verdict cases](tests/it/pgbouncer_benchmark.rs).
- Gate: `cargo test --locked -p pgpool --all-features`

The local harness uses 64 clients, 16 backends, and the simple query protocol. Its ABBA order runs each competitor first and second. A valid result requires all clients to finish and error checks to pass. The local test gate checks harness rules. It does not run the performance comparison. The enforced isolated meter ratchet remains open. A ratchet is a gate that rejects a regression.

### EC Gates Configured

- ID: `ec-gates-configured`
- Promise: Keep the service evidence contract inventory explicit. Supply vat meter and guard runners with evidence under `external-contracts/`.
- Sources:
  - `external:pgpool` declares required service traits in [aw.toml](aw.toml) and checks the current local [benchmark contract](tests/it/pgbouncer_benchmark.rs).
- Gate: `cargo test --locked -p pgpool --all-features`

EC means evidence contract. The local gate checks only existing test and harness contracts. The full inventory, vat runners, and external evidence gates remain open. This capability is a retained target.

### HTTP/2 API List

- ID: `http2-api-list`
- Promise: Publish standard admin routes plus `/pools`, `/pools/{pool}/stats`, and `/drain`. Keep the served OpenAPI contract equal to the offline spec.
- Sources:
  - `external:pgpool` defines the [offline spec](src/interfaces/spec.rs), registers [admin routes](src/interfaces/admin/router.rs), and compares both in [admin cases](tests/it/admin_plane.rs).
- Gate: `cargo test --locked -p pgpool --all-features`

Use `pgpool spec --format routes` for the route list. Use `pgpool spec --format openapi` for the offline twin of `/openapi.json`. The served checks are local behavior evidence. They do not prove deployment or release acceptance.

### Kubernetes-Native Deployment

- ID: `kubernetes-native-deployment`
- Promise: Ship CRD, operator, and instance render commands. Use stateless Deployment, ClusterIP, and PDB assets. Admit endpoint quotas before apply and drain before release.
- Sources:
  - `external:pgpool` renders [operator assets](src/interfaces/operator/render.rs) and [instance profiles](src/application/k8s/instance.rs), applies [quota and drain decisions](src/application/k8s/control.rs), and checks [operator behavior](tests/it/operator.rs).
- Gate: `cargo test --locked -p pgpool --all-features`

The namespaced Pgpool CRD uses the existing leader-elected operator and service-k8s mechanics. Instance profiles cover dev, staging, prod, and template. PostgreSQL owns durable data. Images, repeatable kind drain checks, and deployment acceptance remain open. Historical kind admission notes remain in STATUS.

### Long-Running Stability

- ID: `long-running-stability`
- Promise: Run without backend connection or file descriptor leaks. Preserve in-flight transactions during drain. Preserve pool state safety through backend restarts and rolling deploys.
- Sources:
  - `external:pgpool` checks bounded reuse in [pool mode cases](tests/it/pool_modes.rs), dropped lease capacity in [pool cases](tests/it/pool.rs), and drain in [admin cases](tests/it/admin_plane.rs).
- Gate: `cargo test --locked -p pgpool --all-features`

The churn case uses 100 cycles. The dropped-lease case checks capacity return without an explicit release. These are bounded local cases. A true long-run soak, restart safety, and rolling deployment acceptance remain open.

### Security Hardening

- ID: `security-hardening`
- Promise: Pass authentication to PostgreSQL without storing credentials. Reject malformed wire frames. Define TLS on frontend and backend links. Gate the admin plane before production acceptance.
- Sources:
  - `external:pgpool` handles auth in the [proxy edge](src/interfaces/proxy.rs), rejects malformed frames in [wire cases](tests/it/wire_codec.rs), and checks managed TLS in [discovery cases](tests/it/connection_discovery.rs).
- Gate: `cargo test --locked -p pgpool --all-features`

The local slice includes auth passthrough, wire validation, and configured managed discovery TLS. Standard probes remain tokenless. Full relay TLS, mutating admin authorization, and guard-security evidence remain open. Discovery TLS does not prove those boundaries.

### Standard Operational Endpoints

- ID: `standard-operational-endpoints`
- Promise: Serve `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, and `/docs` on one admin port. Flip readiness during drain. Keep `pgpool spec` as the offline twin.
- Sources:
  - `external:pgpool` registers [standard routes](src/interfaces/admin/router.rs), shares drain in [admin wiring](src/interfaces/admin/wiring.rs), and checks probes and gauges in [admin cases](tests/it/admin_plane.rs).
- Gate: `cargo test --locked -p pgpool --all-features`

The default admin bind is `0.0.0.0:9080` through `RuntimePlan.admin_bind`. The shared server-http listener supports h2c and HTTP/1.1. Metrics use metrics-prometheus. Local cases do not certify a production cluster.

## Supporting documents

- [STATUS.md](STATUS.md) states current support and preserves unverified history.
- [ROADMAP.md](ROADMAP.md) keeps open outcomes and non-goals.
- [CONTRIBUTING.md](CONTRIBUTING.md) states change and check rules.
- [Document index](docs/README.md) maps the architecture pages.
- [Architecture](docs/architecture.md) maps code, public paths, and debt.
- [Domain page](docs/domain/pgpool.md) explains pool ownership and boundaries.
- [Glossary](docs/glossary.md) defines the terms used here.
- [Layout decision](docs/adr/0001-standard-layout-and-ddd.md) records the choice.
- [Architecture conformance](docs/operations/architecture-conformance.md) gives the required checks.
