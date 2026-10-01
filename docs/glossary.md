# Glossary

| Term | Meaning in pgpool |
|---|---|
| Admission | The decision to accept a client or grant backend capacity. |
| Backend | A PostgreSQL connection used by pgpool for a client. |
| Frontend | A client connection accepted by pgpool. |
| Session pooling | One backend stays with a client session until that session ends. |
| Transaction pooling | A backend stays with a transaction, then returns through reset for reuse. |
| Reset | Clear backend session state before another owner uses it. |
| Drain | Stop new admission and let active work finish within the timeout. |
| Reactor | The readiness loop that owns transaction socket state and drives input and output. |
| Budget | A limit on accepted frontend or opened backend connections. |
| Quota | A backend capacity share admitted for a pod at one endpoint. |
| Reserve lease | A temporary capacity grant with expiry and release. |
| Endpoint | A PostgreSQL backend address, provider, and role. |
| Discovery | A live PostgreSQL query that collects capacity facts. |
| Advisory limit | A configured provider limit combined with runtime facts. |
| Context | One product boundary with its model and operations. |
| Domain | Pure values, rules, and deterministic state. |
| Application | Operations that drive the model and service behavior. |
| Infrastructure | Code that connects to live technology. |
| Interface | An input or output edge, such as HTTP or the operator. |
| Assembly | Code that connects layers into the process or public API. |
| Facade | Re-exports that preserve a public name after source moves. |
| ADR | Architecture decision record. It states a choice and limits. |
| B3 | The architecture rule for layer dependency direction. |
| C1 | A warning above 400 measured source lines. It cannot be suppressed. |
| C2 | An error above 1,000 measured source lines. |
| h2c | HTTP/2 without TLS. |
| EC | Evidence contract. It names behavior and its executable proof. |
| ABBA | A comparison order that runs each competitor first and second. |
| Ratchet | A gate that rejects a regression from its accepted baseline. |
| P1 | This layout change that preserves paths and behavior. |
| P2 | Core's later API and architecture changes. Pgpool does not adopt them in this change. |

[Architecture](architecture.md) maps these terms to code.
[STATUS.md](../STATUS.md) states current limits.
