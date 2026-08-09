# mcb-domain::utils — Utilities Policy

> Canonical location for cross-crate utility code in the MCB workspace.

## What Belongs Here

Utilities that are used by **two or more crates** and operate on **domain-agnostic data**:

| Module | Responsibility |
| -------- | --------------- |
| `utils::path` | Workspace-relative paths, slash normalization, strict `strip_prefix` |
| `utils::time` | Epoch timestamps (seconds, nanos), trace-ID seeds |
| `utils::id` | Stable ID hashing (`HMAC-SHA256`), content hashing, ID masking |
| `utils::env` | Strict environment variable parsing — `required_env`, typed parsers |

**Every new module must be justified and documented in this table before merging.**

## Dependency Policy

`mcb-domain` is the **innermost** crate. It MUST NOT depend on other workspace crates.

Allowed external dependencies:

- Pure libraries (`sha2`, `hmac`, `hex`, `chrono`, `uuid`, `regex`, etc.)
- Async/IO libraries **only when necessary** (`tokio`, `reqwest` — user-approved)
- Serialization (`serde`, `serde_json`)

Prohibited:

- Any `mcb-*` workspace crate dependency
- Web framework types (`axum`, `actix`, `rmcp`)
- Database types (`sqlx`, `diesel`)

## Naming Conventions

| Pattern | Name Style | Returns |
| --------- | ----------- | --------- |
| Fallible operation | `parse_*`, `try_*`, `resolve_*`, `require_*` | `Result<T, Error>` |
| Infallible getter | `get_*`, `compute_*` | `T` (only if truly infallible) |
| Validation (no mutation) | `validate_*` | `Result<(), Error>` |
| Canonicalization (fallible, mutation) | `canonicalize_*` | `Result<T, Error>` |

**Avoid**: `normalize_*` for fallible functions — it implies infallibility.

## Strictness Policy — No Fallbacks (HARD RULE)

**Every utility MUST fail fast on invalid inputs. No silent recovery.**

### Banned Patterns

| Pattern | Why It's Banned | Replacement |
| --------- | ---------------- | ------------- |
| `to_string_lossy()` on paths | Silently corrupts non-UTF8 paths | Return `Error` if path is not valid UTF-8 |
| `unwrap_or(0)` / `map_or(0, ..)` for timestamps | Hides clock failures | Return `Result<T>` |
| `unwrap_or_default()` for critical data | Substitutes empty/zero for failures | Return `Result<T>` |
| `canonicalize(..).unwrap_or_else(\|_\| original)` | Hides filesystem errors | Return `Result<PathBuf>` |
| `strip_prefix(..).unwrap_or(absolute)` | Silently returns absolute when relative expected | Return `Error` |
| `env::var(..).unwrap_or("default")` for critical settings | Hides missing config | Use `require_env()` or explicit `expect()` with message |

### When Defaults ARE Acceptable

Defaults are acceptable **only** for genuinely optional, non-critical settings where the default is documented and intentional:

- Log level defaulting to `info`
- HTTP port defaulting to `8080`
- Optional feature flags

Even then, prefer explicit `Option<T>` over silent defaults.

## No Wrappers Policy (HARD RULE)

**Callsites MUST call canonical `mcb_domain::utils::*` methods directly.**

Prohibited:

```rust
// ❌ Wrapper in mcb-server that just forwards
pub fn hash_id(kind: &str, raw_id: &str) -> String {
    mcb_domain::utils::compute_stable_id_hash(kind, raw_id, get_secret())
}
```

Required:

```rust
// ✅ Callsite uses domain directly
let hashed = mcb_domain::utils::id::compute_stable_id_hash(kind, raw_id, secret)?;
```

If a callsite needs environment-specific configuration (like reading a secret from env), that configuration lookup happens at the callsite — not hidden inside a wrapper.

## Anti-Patterns

1. **`helpers.rs` sprawl** — Don't dump unrelated functions into a single `helpers.rs`. Use specific modules (`path`, `time`, `id`, `env`).
2. **Duplicate timestamp functions** — Only one `epoch_secs()` implementation should exist in the workspace (in `utils::time`).
3. **Silent fallbacks** — See "Banned Patterns" above.
4. **Re-exports that hide origin** — `pub use helpers::*` makes it unclear where functions come from. Prefer explicit module paths.
