# Pgpool roadmap

## Purpose

Keep existing open outcomes separate from current support.
These groups do not set an owner, date, or approved delivery order.
Tracking links lead to retained [historical notes](STATUS.md#evidence-policy).
Those notes were not checked against a live tracker.

## Near-term outcomes

### Provider adapter conformance

- ID: `provider-adapter-conformance`
- Outcome: Prove the endpoint and authentication seam for supported provider adapters.
- Boundary: Keep provider SDKs and authentication outside the pooler core. Keep Cloud SQL, AlloyDB, and plain PostgreSQL discovery roles explicit.
- Completion evidence: Runnable adapter and authentication cases that preserve the core runtime boundary.
- Tracking: [Unverified historical notes](STATUS.md#evidence-policy).

### External contract gates

- ID: `external-contract-gates`
- Outcome: Make the required EC inventory and vat meter and guard runners executable.
- Boundary: Keep evidence contracts under external-contracts. Local Rust tests remain one part of the evidence.
- Completion evidence: An explicit inventory, resolvable runners, and retained passing evidence for every required external contract.
- Tracking: [Unverified historical notes](STATUS.md#evidence-policy).

### Pooler security evidence

- ID: `pooler-security-evidence`
- Outcome: Prove credential handling, relay TLS posture, malformed frame rejection, and admin authorization.
- Boundary: Keep standard probes tokenless. Gate mutating admin operations. Discovery TLS covers only discovery.
- Completion evidence: Required negative cases and a guard-security run for the pooler and admin surface.
- Tracking: [Unverified historical notes](STATUS.md#evidence-policy).

### Kubernetes release evidence

- ID: `kubernetes-release-evidence`
- Outcome: Complete images and repeatable Kubernetes admission, readiness, and drain checks.
- Boundary: Keep pgpool stateless. Preserve CRD, operator, instance, quota, and drain contracts. Historical kind admission is an old note.
- Completion evidence: Candidate image evidence and repeatable cluster cases for rendered artifacts and graceful drain.
- Tracking: [Unverified historical notes](STATUS.md#evidence-policy).

## Later outcomes

### Competitor feature parity

- ID: `competitor-feature-parity`
- Outcome: Prove the functions needed for PgBouncer, Odyssey, and pgcat replacement for Axiom workloads.
- Boundary: Cover transaction and session pooling, bounded admission, drain, and measurements. Keep the extended-protocol limit explicit until it changes.
- Completion evidence: A complete external parity matrix with executable cases for every claimed replacement boundary.
- Tracking: [Unverified historical notes](STATUS.md#evidence-policy).

### Isolated performance ratchet

- ID: `isolated-performance-ratchet`
- Outcome: Promote throughput and latency checks to an enforced isolated meter gate.
- Boundary: Keep fixed local ABBA comparisons advisory until promotion. Retain complete-client and error checks for the 64-client, 16-backend simple-protocol profile.
- Completion evidence: Executable vat configuration and isolated results that pass the enforced regression rule.
- Tracking: [Unverified historical notes](STATUS.md#evidence-policy).

### Long running stability evidence

- ID: `long-running-stability-evidence`
- Outcome: Prove no connection or file descriptor leaks through a true long-run soak.
- Boundary: Include in-flight drain safety, backend restarts, and rolling deployment behavior. Bounded 100-cycle cases remain partial evidence.
- Completion evidence: Runnable long-run, drain, restart, and rolling deployment gates with retained results.
- Tracking: [Unverified historical notes](STATUS.md#evidence-policy).

### Default serve entrypoint

- ID: `default-serve-entrypoint`
- Outcome: Resolve the existing serve-by-default process outcome.
- Boundary: The current parser requires a command. Any default change must preserve offline commands and receive its own behavior review.
- Completion evidence: An approved CLI contract and compiled-binary cases for the chosen no-command behavior.
- Tracking: [Unverified historical notes](STATUS.md#evidence-policy).

## Non-goals

### Pooler owned storage

- ID: `pooler-owned-storage`
- Reason: PostgreSQL owns durable data. Pgpool owns admission, reuse, budgets, drain, and operations.

### Required provider SDK

- ID: `required-provider-sdk`
- Reason: Provider connectors stay explicit adapters. The pooler core must not require a platform SDK.
