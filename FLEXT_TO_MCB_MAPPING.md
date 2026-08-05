<!-- markdownlint-disable MD013 -->
# FLEXT → MCB Python Mapping

This document maps [FLEXT](https://github.com/marlonsc/flext) primitives to the
names and conventions used in MCB's Python tooling under `scripts/`.

## Result container

| FLEXT | MCB | Notes |
|-------|-----|-------|
| `flext_core.FlextResult[T]` | `lib.core.r[T]` / `McbResult[T]` | Imported as `r` for brevity. |
| `.success` / `.failure` | same | Boolean status. |
| `.unwrap()` | same | Returns value or raises `RuntimeError`. |
| `.error` | same | `str \| None` message. |
| `.ok(value)` / `.fail(msg)` | same | Constructors. |

**Good:**

```python
from lib.core import r

def parse(path: Path) -> r[list[Issue]]:
    if not path.exists():
        return r[list[Issue]].fail(f"not found: {path}")
    return r[list[Issue]].ok(_do_parse(path))
```

**Bad:**

```python
def parse(path: Path) -> list[Issue]:
    if not path.exists():
        print(f"not found: {path}")
        return []
```

## Settings

| FLEXT | MCB | Notes |
|-------|-----|-------|
| `FlextSettingsBase` | `lib.settings.BaseMcbSettings` | `MCB_` env prefix. |
| — | `lib.settings.BaseCommandSettings` | No env prefix; for `cosmos-command` scripts. |
| — | `lib.settings.McbSettings` | Shared MCB paths (`project_root`, `k8s_dir`, `docs_dir`, `qlty_*`). |
| `.model_config` | same | Pydantic `SettingsConfigDict`. |

**Good:**

```python
from lib.core import BaseCommandSettings, get_logger, r
from pydantic import Field

class GitopsSettings(BaseCommandSettings):
    root: Path = Field(default=Path("."), description="Project root")
```

**Bad:**

```python
ROOT = Path(__file__).resolve().parents[2]  # hardcoded traversal
```

## Logging

| FLEXT | MCB | Notes |
|-------|-----|-------|
| `FlextLogger.fetch_logger(name)` | `lib.core.get_logger(name)` | Returns structlog logger. |

**Good:**

```python
logger = get_logger(__name__)
logger.info("validated", count=len(items))
```

**Bad:**

```python
print("validated", len(items))
```

## Service base

| FLEXT | MCB | Notes |
|-------|-----|-------|
| `FlextService` | `lib.core.McbScriptsService` | For stateful scripts that need lifecycle. |

Use when a script holds clients, caches, or background resources. Most
command-line scripts do not need it.

## CLI wiring

MCB uses `lib.cli.create_app_with_common_params` and
`lib.cli.register_result_command` to expose a handler that returns `r[T]`.

**Good:**

```python
from lib.cli import create_app_with_common_params, register_result_command

def run(settings: MySettings) -> r[int]:
    ...

def main() -> None:
    app = create_app_with_common_params(name="my-cmd", help_text="...")
    register_result_command(app, name="run", help_text="...", model_cls=MySettings, handler=run)
    app()
```

## Import order

Groups, separated by a blank line:

1. `from __future__ import annotations`
2. `stdlib` (`pathlib`, `subprocess`, ...)
3. Third-party (`typer`, `pydantic`, `flext_core`)
4. `lib.*` helpers
5. Project subpackages (`qlty.*`, `docs.py.*`)

Run `ruff check --select I` to verify.

## Testing

| Practice | Example |
|----------|---------|
| Unwrap results | `result = fn(); assert not result.failure; value = result.unwrap()` |
| Use pytest | `pytest scripts/lib/tests` |
| Avoid `print` / `sys.exit` in handlers | Return `r[T]` and let CLI handle exit codes |

## Banned patterns

Enforced by `make check WHAT=guard`:

- `unwrap()`, `expect()`, `panic!`, `todo!`, `unimplemented!` in production paths
- `TODO` / `FIXME` comments
- `print()` / `sys.exit()` in handler logic
- Hardcoded paths outside `McbSettings`
