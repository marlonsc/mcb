<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->

# FLEXT to MCB Pattern Mapping

**Purpose:** Translate foundational FLEXT workspace patterns to the MCB Rust workspace so agents and contributors familiar with FLEXT can ramp up quickly.

**Sources:**

- FLEXT: the published `flext-sh` repositories on branch `0.12.0-dev` (`flext-core`, `flext-cli`, `flext-tests`), consumed as pinned git dependencies — never a local checkout
- MCB: `AGENTS.md`, `Makefile`, `Cargo.toml`, `docs/architecture/PATTERNS.md`

## High-level analogy

| FLEXT | MCB |
|-------|-----|
| Python 3.13 + Pydantic v2 multi-package workspace | Rust 2024 + SeaQL/Loco.rs multi-crate workspace |
| `flext-core` — foundation framework | `mcb-domain` + `mcb-utils` — core types and utilities |
| `flext-cli` — developer CLI | `mcb` — CLI facade binary |
| `flext-tests` — shared test utilities | `tests/` directories inside each `crates/mcb-*/` |
| `flext-quality` / `flext-infra` | `mcb-validate` / `scripts/lib/mcb.sh` |
| `base.mk` + generated Makefiles | `Makefile` + `makefiles/dispatch.mk` + `scripts/lib/mcb.sh` |

## Result flow

### FLEXT

```python
from flext_core import r

def load(user_id: int) -> r[m.User]:
    ...
```

### MCB

```rust
use mcb_domain::Result;

async fn load(user_id: SessionId) -> Result<User> {
    ...
}
```

**Rule:** Fallible paths return typed `Result`. Propagate with `?`.

## Error construction

### FLEXT

```python
raise e.ValidationError("invalid integer") from exc
```

### MCB

```rust
use mcb_domain::Error;

return Err(Error::validation("invalid integer"));
```

**Rule:** Use factory methods; never construct raw variants.

## Logging

### FLEXT

```python
from flext_core import u

logger = u.get_logger(__name__)
logger.info("user.created", user_id=user_id)
```

### MCB

```rust
use tracing::info;

info!(user_id = %user_id, "user.created");
```

**Rule:** Use structured logging (`tracing`); no `print`/`println` in production.

## Import boundaries

### FLEXT

```python
from __future__ import annotations
from collections.abc import Mapping, Sequence
from pathlib import Path

from pydantic import BaseModel
from flext_core import c, m, r, t, u
```

### MCB

```rust
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_utils::id::generate_id;

use crate::config::AppConfig;
```

**Rule:**

- FLEXT: stdlib → third-party → `flext_core` aliases.
- MCB: `std` → external crates → `mcb_*` crates → `crate::`.

## Architecture layers

### FLEXT

```text
constants/typings → runtime → protocols → models → utilities → logging/container → dispatcher
```

### MCB

```text
mcb-domain → mcb-providers → mcb-infrastructure → mcb-server
mcb-utils (leaf, no mcb-* deps)
mcb-validate (developer tooling)
```

**Rule:** Dependencies point inward. Outer crates use ports; inner crates define them.

## Quality gates

### FLEXT

```bash
ruff check <file>
pyrefly check <file>
pytest
```

### MCB

```bash
make check WHAT=fix ACT=fmt APPLY=Y   # rustfmt
make check WHAT=lint                   # fmt + clippy
make test                              # cargo test / nextest
make check WHAT=validate QUICK=1       # architecture validation
make check WHAT=guard                  # banned-pattern scan
```

**Rule:** Run fast gates first, then broad gates. A red gate blocks progression.

## Testing

### FLEXT

```python
def test_load_user():
    result = load(1)
    assert result.is_ok
```

### MCB

```rust
#[tokio::test]
async fn load_user_succeeds() {
    let user = load(SessionId::new()).await.unwrap();
    assert_eq!(user.name, "alice");
}
```

**Rule:**

- FLEXT: public-API-only assertions, real-flow-over-mocks, AAA structure.
- MCB: tests in `tests/`, `rstest` for parameters, `mockall` for mocks, `insta` for snapshots, `extern crate mcb_providers;` for linkme registration.

## Settings / configuration

### FLEXT

```python
from pydantic_settings import BaseSettings

class FlextSettings(BaseSettings):
    env: str = "dev"
```

### MCB

```rust
// config/development.yaml under settings:
providers:
  embedding:
    provider: openai
server:
  network:
    port: 3000
```

**Rule:** MCB uses Loco environment-based YAML files. Environment variables override files. No hardcoded config in code.

## Provider registration

### FLEXT

```python
# Inheritance/MRO-based plugin discovery via project facades
```

### MCB

```rust
impl_registry!(embedding, EmbeddingProvider, EmbeddingConfigContainer);

#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static OPENAI: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "openai",
    description: "...",
    factory: openai_embedding_factory,
};
```

**Rule:** MCB uses compile-time `linkme` distributed slices + runtime `Handle<T>` for provider discovery and switching.

## What to avoid

| FLEXT anti-pattern | MCB anti-pattern |
|--------------------|------------------|
| Bare `except:` | `unwrap()`/`expect()`/`panic!()` in prod |
| `print()` in `src/` | `println!()`/`eprintln!()` in prod |
| Raw dict error envelopes | Raw `Error::ProviderError { ... }` construction |
| Direct framework imports (pydantic, structlog) in consumers | Concrete provider imports in handlers |
| `TODO`/`FIXME`/`HACK`/`XXX` | `TODO`/`FIXME` markers and `todo!()` in prod |
| Wildcard imports | `use mcb_domain::*;` outside `lib.rs` |

## Skill mapping

| FLEXT skill | MCB skill | Purpose |
|-------------|-----------|---------|
| `flext-patterns` | `mcb-patterns` | Central index |
| `flext-quality-gates` | `mcb-quality-gates` | Gate sequence |
| `flext-import-rules` | `mcb-import-rules` | Import hygiene |
| `flext-architecture-layers` | `mcb-architecture-layers` | Crate boundaries |
| `flext-strict-typing` | `mcb-error-handling` | Typed errors / logging |
| `testing-patterns` | `mcb-testing-patterns` | Test discipline |
| `flext-development-workflow` | `mcb-make-verbs` | Make verbs |

## See also

- `.agents/skills/mcb-patterns/SKILL.md` — central MCB index
- `docs/architecture/PATTERNS.md` — detailed MCB patterns
- `docs/developer/CONTRIBUTING.md` — MCB contribution guide
- <https://github.com/flext-sh/flext-core/tree/0.12.0-dev> — FLEXT canonical law on the published branch
