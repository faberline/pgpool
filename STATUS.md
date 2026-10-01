# Pgpool status

## Scope

This page describes current source support.
It does not state that the product is released or accepted for production.
A listed gate is a required command.
This page does not claim that a gate ran in the current session.

## State definitions

| State | Meaning |
|---|---|
| Supported | Current source provides the stated scope and has a named local gate. |
| Limited | Current source provides the stated scope, with an open material boundary. |
| Not supported | The full surface has no current executable support claim. |

## Support matrix

| Surface | ID | State | Supported scope | Limits | Evidence |
|---|---|---|---|---|---|
| Working-Name App Scaffold | `working-name-app-scaffold` | Supported | Root package, binary name, offline plan, and project record. | The final product name is open. | `cargo test --locked -p pgpool --all-features` |
| Shared Server Substrate Adoption | `shared-server-substrate-adoption` | Supported | Shared TCP, HTTP, budget, and drain composition. | Core dependencies stay at v0.4.14. | `cargo test --locked -p pgpool --all-features` |
| PostgreSQL Pooler Core | `postgresql-pooler-core` | Limited | Bounded session pooling and simple-protocol transaction pooling. | Transaction mode rejects extended queries. [Full replacement breadth](ROADMAP.md#competitor-feature-parity). | `cargo test --locked -p pgpool --all-features` |
| Platform Adapter Boundary | `platform-adapter-boundary` | Limited | Provider and role values, live capacity discovery, and configured discovery TLS. | Provider authentication and full seam checks remain open. [Adapter conformance](ROADMAP.md#provider-adapter-conformance). | `cargo test --locked -p pgpool --all-features` |
| CLI Interface | `cli-interface` | Limited | Explicit serve, runtime-plan, spec, and Kubernetes commands. | The parser requires a command. [Default serve outcome](ROADMAP.md#default-serve-entrypoint). | `cargo test --locked -p pgpool --all-features` |
| CLI Standard Surface | `cli-standard-surface` | Supported | Local help and routing for llm, upgrade, and issue with build data. | Online commands need the matching feature and external access. Local help is not online acceptance. | `cargo test --locked -p pgpool --all-features` |
| Chainable Output Conformance | `chainable-output-conformance` | Supported | Runtime-plan next marker and raw spec or manifest bytes. | Consumers must separate plan JSON from the next marker. | `cargo test --locked -p pgpool --all-features` |
| Competitor Feature Parity | `competitor-feature-parity` | Limited | Current local pooling, drain, and admin cases. | The full external matrix remains open. [Parity outcome](ROADMAP.md#competitor-feature-parity). | `cargo test --locked -p pgpool --all-features` |
| Competitor Performance | `competitor-performance` | Limited | Local 64-client, 16-backend simple-protocol ABBA harness and verdict rules. | The local gate checks harness rules only. [Isolated performance ratchet](ROADMAP.md#isolated-performance-ratchet). | `cargo test --locked -p pgpool --all-features` |
| EC Gates Configured | `ec-gates-configured` | Not supported | No complete external evidence suite is claimed. | The full inventory, vat runners, and evidence gates remain open. | [Required external gates](ROADMAP.md#external-contract-gates). |
| HTTP/2 API List | `http2-api-list` | Supported | Offline inventory and served OpenAPI twin on the local admin plane. | Local contract checks do not prove deployment acceptance. | `cargo test --locked -p pgpool --all-features` |
| Kubernetes-Native Deployment | `kubernetes-native-deployment` | Limited | Layered renderers, quota decisions, and drain status models. | Images and repeatable cluster drain acceptance remain open. [Kubernetes evidence](ROADMAP.md#kubernetes-release-evidence). | `cargo test --locked -p pgpool --all-features` |
| Long-Running Stability | `long-running-stability` | Limited | Bounded 100-cycle reuse, dropped-lease capacity, and local drain cases. | A long-run soak and restart or rolling deployment acceptance remain open. [Stability evidence](ROADMAP.md#long-running-stability-evidence). | `cargo test --locked -p pgpool --all-features` |
| Security Hardening | `security-hardening` | Limited | Auth passthrough, malformed frame checks, and configured discovery TLS. | Full relay TLS, mutating admin authorization, and guard-security acceptance remain open. [Security evidence](ROADMAP.md#pooler-security-evidence). | `cargo test --locked -p pgpool --all-features` |
| Standard Operational Endpoints | `standard-operational-endpoints` | Supported | One-port probes, metrics, OpenAPI, docs, and readiness drain behavior. | Local route checks do not certify a production cluster. | `cargo test --locked -p pgpool --all-features` |

## Evidence policy

Current Rust source defines implementation behavior.
Test files name local checks.
Some cases return early when PostgreSQL or a TLS fixture is absent.
A zero test exit alone does not prove those cases ran against that backend.

The local package gate checks code and local contracts.
It does not prove full competitor parity, a performance win, a security review, a long-run soak, or a cluster release.
Historical benchmark wins are not a current performance result.
Historical cluster admission is not a current drain or release result.
No live issue, pull request, release, or deployment state was checked for this page.

The layout gate compares complete test lists and artifact bytes.
The controller owns that comparison and final acceptance.
See [architecture conformance](docs/operations/architecture-conformance.md) for the commands.

**Historical status notes**

The following snapshot preserves the complete prior README.
Its work-item numbers, tables, evidence notes, pending claims, and source paths were not rechecked against a live tracker.
The snapshot records the old document as it was.
Use the support matrix above and current Rust paths for present behavior.

The snapshot retains the note about six eligible local PgBouncer comparison wins.
It also retains the old kind API-server admission note.
Neither note proves a current release or production result.

<details>
<summary>Prior README snapshot with unverified work-item and evidence notes</summary>

~~~markdown
# Pgpool

## Brief

`pgpool` is the working app id for Axiom's Kubernetes-native PostgreSQL
connection pooler. The final product name is not settled yet; until then the
repository path, crate name, binary name, and tracker label use `pgpool` as a
stable implementation placeholder.

The app owns the database-pooling data plane: frontend PostgreSQL TCP
admission, backend connection budgeting, drain-aware shutdown, pool health, and
standard operational HTTP endpoints. Cloud SQL, AlloyDB, and other platform
connectors stay explicit adapters above this core rather than being baked into
the pooler runtime.

Current implementation slice: `pgpool` is a Rust workspace crate and
binary with PostgreSQL wire handling, bounded session/transaction pooling, a
single-owner dense-buffer readiness reactor for the transaction data plane, a
served admin plane, live remote-PostgreSQL capacity discovery, global endpoint
quota/drain models, and layered Pgpool CRD/operator/instance artifacts. Shared
runtime dependencies remain wired through `server-lifecycle`, `server-tcp`,
`server-http`, `metrics-prometheus`, and `service-k8s`; provider authentication
and broader external EC gates remain separate work roots.

## Boundaries

- `pgpool` owns Postgres-compatible pooling and proxy admission.
- `server-lifecycle`, `server-tcp`, and `server-http` own generic server runtime
  mechanics; `pgpool` composes them instead of duplicating accept loops,
  connection budgets, h2c serving, drain, or tracing.
- Platform adapters such as Cloud SQL Proxy or AlloyDB endpoint discovery stay
  optional integration layers, not required runtime dependencies.
- Application services should connect to `pgpool` over the PostgreSQL wire
  protocol and inspect operations through the admin HTTP surface.

## Capabilities

A promise with no gate under it is not claimed.

The baseline capabilities selected by aw.toml's `service` umbrella profile
(plus `cli_facing`, `competitive_replacement`, `kubernetes_native`,
`long_running`, and `network_exposed`) are mandatory for this pooler class.
They do not replace pgpool's product capabilities; the PostgreSQL pooler core
and the platform adapter boundary remain first-class domain roots.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Working-Name App Scaffold | - | crate/bin/README/AW metadata and route inventory are present under `pgpool` |
| Shared Server Substrate Adoption | - | runtime plan composes `server-lifecycle`, `server-tcp`, and `server-http` types |
| PostgreSQL Pooler Core | 1282 | domain: frontend pg wire parser, backend pool, transaction/session modes |
| Platform Adapter Boundary | 1283 | live PostgreSQL capacity discovery is provider/role typed; provider auth remains outside core runtime |
| CLI Interface | 1282 | mandatory baseline: single `pgpool` bin with runtime-plan/spec verbs; serve entrypoint remains open |
| CLI Standard Surface | - | mandatory baseline: shared `cli-std` llm/upgrade/issue surface with build-stamp provenance |
| Chainable Output Conformance | - | mandatory baseline: `runtime-plan` emits `next:`; raw spec streams stay unwrapped |
| Competitor Feature Parity | 1285 | mandatory baseline: PgBouncer/Odyssey/pgcat transaction-pooling replacement breadth |
| Competitor Performance | 1285 | fixed local ABBA harness has six eligible pgpool wins vs PgBouncer (#1753); vat-isolated ratchet remains open |
| EC Gates Configured | 1285 | mandatory baseline: aw.toml EC inventory, vat meter/guard runners, external-contracts evidence |
| HTTP/2 API List | 1282 | mandatory baseline: offline `pgpool spec` admin route inventory; served contract remains open |
| Kubernetes-Native Deployment | 1284 | PgpoolSpec CRD/operator/instance render, shared Deployment composition, quota admission, and drain behavior are covered; image artifact work remains |
| Long-Running Stability | 1282 | mandatory baseline: backend reuse without leaks, graceful drain, restart safety |
| Security Hardening | 1286 | mandatory baseline: frontend auth passthrough, TLS posture, admin-plane exposure gates |
| Standard Operational Endpoints | 1282 | mandatory baseline: one-port `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`; offline twin exists |

### Working-Name App Scaffold

Hold `pgpool` as a stable working app id — crate, binary, README capability
map, and AW metadata live under `pgpool` — so pooler work roots can land
before the final product name is settled, without renaming churn.

- Root WI: none; this capability predates the tracker.
- Surfaces: CLI: `pgpool runtime-plan` - offline shared-runtime plan for the
  data and admin planes.; Config: `aw.toml` - project registration,
  capability profile traits, and workspace test gate.
- Gate — behavior: `cargo test -p pgpool --test cli_contract` - compiled-binary
  contract for the scaffold surface
- Source: `tests/cli_contract.rs`, `aw.toml`,
  `src/bin/pgpool.rs`
- Evidence: tests/cli_contract.rs

### Shared Server Substrate Adoption

`pgpool` starts from the shared service substrate instead of inventing a local
accept loop or HTTP admin server. The TCP data-plane listener uses `server-tcp`
concepts, the admin listener uses `server-http`/h2c concepts, and connection
limits/readiness/drain are represented by `server-lifecycle`.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `RuntimePlan` - composes `server-lifecycle`
  bind/budget/drain, `server-tcp` socket options, and `server-http` h2c
  options.; CLI: `pgpool runtime-plan` - JSON plan naming the shared libs.
- Gate — behavior: `cargo test -p pgpool` - runtime plan composes shared
  substrate types instead of local reinventions
- Source: `src/lib.rs`, `tests/cli_contract.rs`
- Evidence: src/lib.rs

### PostgreSQL Pooler Core

Provide a high-throughput PostgreSQL pooler with bounded frontend admission,
backend connection reuse, transaction/session pool modes, graceful drain, and
clear observability before platform-specific adapters are added.

- Root WI: #1282
- Surfaces: TCP: `0.0.0.0:6432` - PostgreSQL wire protocol frontend admission
  for application clients.; Rust API: `RuntimePlan` - pool mode, frontend
  budget, and backend budget configuration.
- Gate — behavior: pending pg wire parser and pool lifecycle conformance gates
  - startup/auth passthrough, transaction/session pooling, drain
- Gate: tests/wire_codec.rs
  (`cargo test -p pgpool --test wire_codec`)
- Gate: tests/session_proxy.rs
  (`cargo test -p pgpool --test proxy --test session_proxy`)
- Gate: tests/pool_modes.rs
  (`cargo test -p pgpool --test pool --test pool_modes`)
- Source: `tests/proxy.rs`, `tests/pool.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| pg-wire-frontend-protocol | epic | 1287 | tests/wire_codec.rs; tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md |
| backend-pool-and-reuse | epic | 1289 | tests/pool.rs; tests/pool_modes.rs; tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md |
| transaction-session-pool-modes | epic | 1289 | tests/pool.rs; tests/pool_modes.rs; tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md |
| transaction-readiness-reactor | change | 1753 | tech-design/logic/p0-dense-buffer-readiness-reactor.md; tests/pool_modes.rs; benchmarks/pgbouncer-transaction-pooling/run.sh |
| serve-entrypoint-and-drain | epic | 1288 | tests/proxy.rs; tests/session_proxy.rs; tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md |

### Platform Adapter Boundary

Keep Cloud SQL Proxy, AlloyDB endpoint discovery, and other platform connectors
as explicit adapters above the pooler core: the core runtime never embeds
platform SDKs, and adapters only supply backend endpoints and auth material
through a stable seam.

- Root WI: #1283
- Surfaces: Rust API: backend endpoint/auth adapter seam - Cloud SQL, AlloyDB,
  and plain-Postgres backends supply endpoints and auth material above the core
  runtime.
- Gate — behavior: pending adapter seam conformance gates - core runtime stays
  adapter-free
- Gate: adapters compose from outside
- Source:
  `tests/connection_discovery.rs - live PostgreSQL runtime discovery integration gate`,
  `src/platform/discovery.rs - provider/role typed adapter seam and runtime-lower-bound logic`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| backend-adapter-seam | epic | 1283 | pending: adapter seam contract tests |
| runtime-connection-limit-discovery | change | 1570 | tests/connection_discovery.rs; src/platform/discovery.rs; tech-design/semantic/pgpool-runtime-connection-limit-discovery.md |

### CLI Interface

Expose pgpool as one runnable binary with a stable process entrypoint — serve
by default once the pooler core lands — plus offline runtime-plan and spec
verbs for agents and operators.

- Root WI: #1282
- Surfaces: CLI: `pgpool` - single bin; `runtime-plan` and `spec` verbs today,
  serve-by-default data/admin plane entrypoint planned.
- Gate — behavior: `cargo test -p pgpool --test cli_contract` - compiled-binary
  help/verb contract
- Source: `tests/cli_contract.rs`, `src/bin/pgpool.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| offline-plan-and-spec-verbs | change | - | tests/cli_contract.rs |
| serve-by-default-entrypoint | epic | 1288 | tests/cli_contract.rs (`help_and_llm_workflow_topic_mention_serve`); src/bin/pgpool.rs |

### CLI Standard Surface

Ship the mandatory shared `cli-std` surface (llm/upgrade/issue) every ecosystem
CLI owes, backed by build-stamp provenance, without blurring it into pgpool's
domain verbs.

- Root WI: none; this capability predates the tracker.
- Surfaces: CLI: `pgpool llm` - offline agent self-doc topics (outline,
  workflow, api, boundaries).; CLI: `pgpool upgrade` - shared self-update and
  `--check` surface through `cli-std`.; CLI:
  `pgpool issue search|view|create|comment` - shared tracker surface scoped to
  `project:pgpool`.
- Gate — behavior: `cargo test -p pgpool --test cli_contract` -
  llm/upgrade/issue appear in the compiled binary help contract
- Source: `src/bin/pgpool.rs`, `tests/cli_contract.rs`,
  `core/cli-std/src`
- Evidence: tests/cli_contract.rs

### Chainable Output Conformance

Keep pgpool's CLI outputs chainable per the CLI convention: raw artifact
streams (spec renders) stay unwrapped bytes, while operational verbs carry
explicit `next:`/terminal markers.

- Root WI: none; this capability predates the tracker.
- Surfaces: CLI: `pgpool spec --format openapi|openapi-yaml|json-schema|routes`
  - raw artifact streams that intentionally stay unwrapped bytes.; CLI:
  `pgpool runtime-plan` - operational output carrying a runnable `next:` step.
- Gate — behavior: `cargo test -p pgpool --test cli_contract` - runtime-plan
  emits `next: pgpool spec --format routes`
- Gate: spec stdout stays raw parseable bytes
- Source: `tests/cli_contract.rs`, `src/bin/pgpool.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| next-marker-on-runtime-plan | change | - | tests/cli_contract.rs |
| raw-spec-streams-stay-unwrapped | change | - | tests/cli_contract.rs |

### Competitor Feature Parity

Cover the baseline connection-pooler functions pgpool needs to replace
PgBouncer, Odyssey, and pgcat for Axiom workloads: transaction and session
pooling, bounded admission, drain, and pool observability.

- Root WI: #1285
- Surfaces: TCP: PostgreSQL wire frontend - transaction/session pooling
  workflows PgBouncer-class poolers cover.; HTTP: admin pool/stats/drain routes
  - operational parity with pooler admin consoles.
- Gate — behavior: pending pooler parity conformance gates - transaction
  pooling, session pooling, drain, and stats parity vs PgBouncer/Odyssey/pgcat
- Source: `pending: parity conformance matrix vs PgBouncer/Odyssey/pgcat`
- Evidence: pending: parity conformance gates

### Competitor Performance

Tie pgpool's performance claims to repeatable pooled-connection throughput and
latency tests under a vat-isolated meter gate, with the external PgBouncer /
Odyssey / pgcat comparison as advisory dogfood until promoted.

- Root WI: #1285
- Surfaces: Harness:
  `benchmarks/pgbouncer-transaction-pooling/run.sh` - fixed
  counterbalanced PgBouncer transaction-pooling comparison.; Meter/Vat: meter
  diagnostics are executable while `vat.toml#meter-perf` remains
  pending for an isolated ratchet.
- Gate — efficiency: fixed 64-client, 16-backend, simple-protocol release ABBA
  comparison with complete-client/error validation
- Gate: pending vat promotion to an enforced ratchet
- Source:
  `tests/pgbouncer_benchmark.rs - hermetic profile/verdict contract`,
  `benchmarks/pgbouncer-transaction-pooling/run.sh - six eligible release wins recorded on #1753, including the default transaction engine`,
  `pending: vat.toml meter-perf promotion to an enforced isolated ratchet`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| vat-meter-throughput-gate | epic | 1285 | pending: vat meter-perf runner |
| external-pooler-comparison | change | 1753 | fixed local ABBA runner; six eligible pgpool wins vs PgBouncer with both orders unanimous |

### EC Gates Configured

Keep pgpool's service-trait EC baseline explicit and runnable: aw.toml owns the
EC inventory, vat owns the meter/guard runners, and external-contracts/ carries
the evidence contracts each gate closes against.

- Root WI: #1285
- Surfaces: Config: `aw.toml` - AW EC inventory and generated
  dispatch commands (pending).; Config: pending `vat.toml` -
  vat-managed meter/guard runners.
- Gate — behavior: pending a phase-1 project at `e2e/` - no
  black-box case exists yet for the pooler capability set
- Source: `pending: aw.toml EC inventory`,
  `pending: vat meter/guard runners and external-contracts evidence`
- Evidence: pending: aw ec gen --verify

### HTTP/2 API List

Publish pgpool's admin HTTP surface as a compact route inventory — standard
operational endpoints plus `/pools`, `/pools/{pool}/stats`, and `/drain` — with
the offline `pgpool spec` twin matching the served contract once the admin
plane runs.

- Root WI: #1282
- Surfaces: CLI: `pgpool spec --format routes|openapi|openapi-yaml|json-schema`
  - offline admin API inventory and OpenAPI twin.; HTTP: served `/openapi.json`
  and admin routes on the running admin plane, matching the offline twin
  byte-for-byte.
- Gate — behavior: `cargo test -p pgpool` - offline route inventory names the
  standard and pool admin endpoints
- Gate: served-vs-offline conformance proven by `tests/admin_plane.rs`
- Gate: tests/admin_plane.rs
  (`served_contract_matches_offline_spec`, AC3)
- Source: `src/spec.rs`, `tests/cli_contract.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| offline-route-and-openapi-inventory | change | - | src/spec.rs; tests/cli_contract.rs |
| served-contract-matches-offline-spec | epic | 1290 | tests/admin_plane.rs (`served_contract_matches_offline_spec`); tech-design/logic/served-admin-plane-with-drain-aware-readiness.md |

### Kubernetes-Native Deployment

Ship pgpool as a Kubernetes-native pooler: CRD/operator/instance render verbs,
image fixtures rendered from the binary, and pod lifecycle behavior (readiness
flip plus graceful drain) proven in a kind smoke path.

- Root WI: #1284
- Surfaces: CLI: `pgpool k8s crd render`, `pgpool k8s operator render|run`, and
  `pgpool k8s instance render` - layered deployment artifact verbs per the
  service CLI convention.; K8s: namespaced Pgpool CRD, leader-elected operator,
  live endpoint discovery plus pre-apply quota admission, instance profiles,
  and shared stateless Deployment/ClusterIP/PDB composition.
- Gate — behavior: `cargo test -p pgpool --test operator --test cli_contract` -
  CRD/operator/instance artifacts and shared Deployment children render
  deterministically from the binary and typed CR
- Source:
  `tests/operator.rs - CRD structural schema, owned stateless render, readiness, budget-status, and operator asset gates`,
  `tests/cli_contract.rs - layered k8s CLI render contract`,
  `src/k8s/control.rs - deterministic quota admission and drain-before-release reconciliation model`,
  `real kind API-server smoke - generated CRD, Pgpool CR, RBAC, and operator Deployment admitted successfully`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| crd-operator-instance-render | epic | 1284 | tests/operator.rs; tests/cli_contract.rs; tech-design/semantic/pgpool-crd-operator-control-plane.md |
| kind-drain-readiness-smoke | epic | 1284 | pending: kind smoke script |
| stateless-deployment-instance | change | 1561 | src/k8s/instance.rs; negative stateful-boundary tests in the same source unit |
| global-endpoint-quota-allocation | change | 1571 | src/k8s/budget.rs; tech-design/semantic/pgpool-global-endpoint-quota-allocation.md |
| drain-safe-control-plane-status | change | 1573 | src/k8s/control.rs; tech-design/semantic/pgpool-drain-safe-control-plane-status.md |
| crd-operator-control-plane | change | 1575 | src/operator; tests/operator.rs; tech-design/semantic/pgpool-crd-operator-control-plane.md |

### Long-Running Stability

Run as a long-lived pooler without leaking backend connections or file
descriptors, dropping in-flight transactions on drain, or corrupting pool state
across backend restarts and rolling deploys.

- Root WI: #1282
- Surfaces: CLI: pending `pgpool` serve process - durable pooler with
  drain-aware shutdown.; TCP/HTTP: frontend admission and admin plane surviving
  backend restarts and rolling deploys.
- Gate — stability: pending long-run and drain conformance gates - backend
  reuse without connection/fd leaks, drain without dropped in-flight
  transactions, restart safety
- Gate: tests/pool_modes.rs
  (`churn_100_cycles_holds_backend_count_stable_no_leak`)
- Gate: tests/pool.rs
  (`dropped_lease_without_explicit_release_does_not_leak_capacity_slot`) —
  bounded-cycle proof, not a true long-run soak
- Source: `pending: drain and backend-restart conformance tests`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| pool-leak-and-reuse-longrun | epic | 1289 | bounded-cycle proof, not a true long-run soak: tests/pool_modes.rs (`churn_100_cycles_holds_backend_count_stable_no_leak`); tests/pool.rs (`dropped_lease_without_explicit_release_does_not_leak_capacity_slot`) |
| drain-and-backend-restart-safety | epic | 1289 | pending: drain conformance tests |

### Security Hardening

Keep pgpool safe as a network-exposed credential-carrying proxy: auth
passthrough without credential persistence, explicit TLS posture on both
frontend and backend links, malformed wire-frame rejection, and a gated admin
plane before production readiness.

- Root WI: #1286
- Surfaces: TCP: PostgreSQL frontend auth passthrough - client credentials
  verified against the backend, never stored.; HTTP: admin plane exposure
  posture - probes stay tokenless, mutating admin verbs gated.; Env: pending
  TLS material configuration for frontend and backend links.
- Gate — security: pending guard scan and negative gates - auth passthrough,
  TLS posture, malformed-frame rejection, admin exposure
- Source: `pending: vat guard-security runner`,
  `pending: auth passthrough and malformed-frame negative tests`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| auth-passthrough-and-tls-posture | epic | 1286 | pending: negative gates |
| guard-static-runtime-evidence | epic | 1286 | pending: vat guard-security runner |

### Standard Operational Endpoints

Expose the standard one-port operational surface the service trait requires —
probes, metrics scrape, live spec, and Swagger UI stay always-on on the admin
port, with readiness flipping on drain and `pgpool spec` as the offline twin.

- Root WI: #1282
- Surfaces: HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs` -
  one-port operational surface, served on `RuntimePlan.admin_bind` via
  `server_http::serve_h2c_with_options`.; CLI: `pgpool spec` - offline OpenAPI
  evidence for the same contract when no server is running.
- Gate — behavior: `cargo test -p pgpool` - offline inventory carries the five
  standard endpoints
- Gate: served conformance proven by `tests/admin_plane.rs`
- Gate: tests/admin_plane.rs (`all_routes_respond_on_h2c_and_http1`
  AC1, `drain_flips_readyz_and_process_exits_cleanly` AC2,
  `metrics_exposes_prometheus_pool_gauges` AC4)
- Source: `src/spec.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| offline-standard-endpoint-inventory | change | - | src/spec.rs |
| served-probes-and-drain-flip | epic | 1290 | tests/admin_plane.rs (`all_routes_respond_on_h2c_and_http1`, `drain_flips_readyz_and_process_exits_cleanly`, `metrics_exposes_prometheus_pool_gauges`); tech-design/logic/served-admin-plane-with-drain-aware-readiness.md |
~~~

</details>
