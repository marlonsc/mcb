# Serde in MCB: Data Contracts, Compatibility, and Boundary Hygiene

Last updated: 2026-02-12  
Scope: project-specific Serde usage, compatibility strategy, and risk controls.  
Cross-reference: `context/external/figment.md`, `context/external/rocket.md`, `context/external/rmcp.md`, `context/external/thiserror.md`.

---

## 1. Why Serde Matters in This Repository

Serde is the primary contract layer between:

- Domain and transport payloads
- Configuration sources and typed config structs
- Internal entities and JSON-facing surfaces
- Test fixtures and behavioral snapshots

In MCB, serialization quality directly affects MCP compatibility, admin API behavior, and operational tooling.

---

## 2. Coverage and Usage Footprint

From codebase search, Serde is used across nearly all core crates. Representative anchors:

- Domain entities and value objects: `crates/mcb-domain/src/entities/*`
- Workflow and state models: `crates/mcb-domain/src/entities/workflow.rs`
- Language support contracts: `crates/mcb-language-support/src/language.rs:9`
- Validation model/config structures: `crates/mcb-validate/src/*`
- Server response types: `crates/mcb-server/src/handlers/**/responses.rs`
- Provider DTOs and external payload adapters: `crates/mcb-providers/src/**`

This is expected: Serde is part of stable boundary infrastructure, not a per-feature utility.

---

## 3. Core Serde Patterns in MCB

### 3.1 Primary derive pattern

Most common shape:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
}
```

This appears broadly in domain and transport types.

### 3.2 Schema-friendly derive pattern

Publicly exposed or validated models frequently include schema metadata:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Plan {
    pub id: String,
    pub status: PlanStatus,
}
```

See `crates/mcb-domain/src/entities/plan.rs`.

### 3.3 Compatibility attributes

Observed and expected attributes:

- `#[serde(rename = "...")]`
- `#[serde(rename_all = "snake_case" | "lowercase")]`
- `#[serde(default)]`
- `#[serde(skip_serializing_if = "Option::is_none")]`
- `#[serde(tag = "...", content = "...")]` for tagged enums

Examples:

- `crates/mcb-domain/src/entities/workflow.rs`
- `crates/mcb-domain/src/entities/observation.rs`
- `crates/mcb-language-support/src/language.rs:17`

### 3.4 JSON value utilities where needed

`serde_json::json!`, `to_value`, `from_value`, `to_string`, and `from_str` appear in provider boundaries and tests.

Guidance in this repository is to keep these operations localized to boundaries and testing concerns.

---

## 4. Architecture Boundaries (Critical)

### 4.1 Allowed in domain

Allowed:

- `serde::{Serialize, Deserialize}` derives on domain entities/value objects.

Not preferred in domain core behavior:

- Serialization-format-specific logic bleeding into business rules.

### 4.2 Boundary handling

Transport-specific shape decisions should stay near server/provider layers, not deep in domain behavior.

Related docs:

- `context/project-intelligence/architecture-boundaries.md`
- `context/project-intelligence/clean-architecture.md`

---

## 5. Compatibility and Evolution Strategy

When evolving payloads/config:

1. Add fields with `#[serde(default)]` where safe.
2. Use `alias` during migration windows if field names change.
3. Keep `rename_all` consistent per type family.
4. Avoid hidden breaking changes in externally consumed JSON.

For enums:

- Prefer explicit tagging for long-lived protocol surfaces.
- Avoid implicit representation switches without migration guidance.

---

## 6. Common Failure Modes

### 6.1 Silent break on field rename

Risk:

- Consumers fail deserialization after rename.

Mitigation:

- Temporary alias support + test coverage + release notes.

### 6.2 Option/default mismatch

Risk:

- Missing field behavior diverges from expected domain default.

Mitigation:

- Centralize defaults and test decode behavior from partial payloads.

### 6.3 Numeric precision expectations

Risk:

- Downstream JavaScript clients lose precision for large integers.

Mitigation:

- Keep large IDs as strings in external contracts when precision risk exists.

### 6.4 Overuse of untyped JSON blobs

Risk:

- Contract drift and weaker compile-time checks.

Mitigation:

- Prefer typed structs; isolate `serde_json::Value` to genuinely dynamic payload sections.

---

## 7. Contributor Do/Do-Not Checklist

Do:

- Use typed structs/enums with explicit attributes.
- Add migration-safe aliases/defaults when evolving contracts.
- Keep serialization concerns near boundaries.
- Add round-trip tests for key contract types.

Do not:

- Rename externally visible fields without migration plan.
- Introduce inconsistent casing/tagging conventions in adjacent APIs.
- Replace typed model contracts with arbitrary `Value` in stable interfaces.

---

## 8. Verification Checklist

When changing Serde-relevant code:

1. Confirm derives and serde attributes are intentional and consistent.
2. Validate old payload compatibility where contracts are long-lived.
3. Add/refresh round-trip tests.
4. Verify no architecture boundary leakage occurred.
5. Verify documentation references the actual field/tag behavior.

Suggested commands:

```bash
rg -n "derive\((.*Serialize|.*Deserialize)" crates
rg -n "#\[serde\(" crates
cargo test
```

---

## 9. Cross-Document Map

- Config-focused serde usage: `context/external/figment.md`
- HTTP/API boundary serialization: `context/external/rocket.md`
- MCP/tool payload contracts: `context/external/rmcp.md`
- Error payload conversion and typed failures: `context/external/thiserror.md`

---

## 10. References

Official:

- https://serde.rs/
- https://docs.rs/serde
- https://docs.rs/serde_json

Repository anchors:

- `crates/mcb-domain/src/entities/plan.rs`
- `crates/mcb-domain/src/entities/workflow.rs`
- `crates/mcb-domain/src/entities/observation.rs`
- `crates/mcb-language-support/src/language.rs`

External examples:

- https://github.com/serde-rs/serde/tree/master/serde_derive
- https://github.com/tokio-rs/axum/tree/main/examples
