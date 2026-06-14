# Serena Configuration for MCB

This document describes the Serena setup for the Memory Context Browser (MCB) project, including configuration, available tools, validated usage patterns, and real-world test results.

## What is Serena?

[Serena](https://github.com/oraios/serena) is a semantic code retrieval and editing toolkit that provides AI assistants with deep code understanding capabilities through Language Server Protocol (LSP) integration. It enables precise symbol navigation, reference finding, and code manipulation.

## Current Configuration

### Version
- **Serena Agent**: 1.5.3 (installed via `uv tool install serena-agent`)
- **Language Backend**: LSP (rust-analyzer for Rust)

### Project Configuration
The MCB-specific configuration lives in `.serena/project.yml`:

| Setting | Value | Rationale |
|---------|-------|-----------|
| `project_name` | `mcb` | Project identifier |
| `languages` | `rust` | Primary language |
| `tool_timeout` | `60` | Large Rust projects need more time |
| `symbol_info_budget` | `15.0` | rust-analyzer docstring resolution |
| `line_ending` | `lf` | Unix convention |
| `ignore_all_files_in_gitignore` | `true` | Respect .gitignore |

### Indexed Files
- **Total Rust files**: 1361
- **Cache location**: `.serena/cache/rust/`
- **Index command**: `serena project index`

## Available MCP Tools (20 total) — All Tested ✅

The following tools are exposed through the MCP interface to AI assistants. **All 20 tools have been tested and confirmed working** on the MCB codebase.

### Navigation & Discovery (5)
| Tool | Status | Avg Time | Description |
|------|--------|----------|-------------|
| **`get_symbols_overview`** | ✅ | ~2s* | High-level symbol map of a file |
| **`find_symbol`** | ✅ | ~2s | Global/local symbol search by name path pattern |
| **`find_declaration`** | ✅ | ~2s | Jump to symbol declaration/definition |
| **`find_implementations`** | ✅ | ~0.2s | Find symbols implementing a trait/interface |
| **`find_referencing_symbols`** | ✅ | ~0.2s | Find all references to a symbol |

\* First call after server start takes ~24s due to rust-analyzer initialization. Subsequent calls are fast.

### Code Analysis (1)
| Tool | Status | Avg Time | Description |
|------|--------|----------|-------------|
| **`get_diagnostics_for_file`** | ✅ | ~2.5s | LSP diagnostics (errors, warnings) for a file |

### Code Editing (6)
| Tool | Status | Avg Time | Description |
|------|--------|----------|-------------|
| **`replace_content`** | ✅ | ~0.1s | Replace text patterns in a file (regex supported) |
| **`replace_symbol_body`** | ✅ | ~0.3s | Replace full symbol definition via LSP |
| **`insert_after_symbol`** | ✅ | ~0.3s | Insert code after a symbol definition |
| **`insert_before_symbol`** | ✅ | ~0.3s | Insert code before a symbol definition |
| **`rename_symbol`** | ✅ | ~21s | LSP-powered rename refactoring across codebase |
| **`safe_delete_symbol`** | ✅ | ~2.5s | Safe symbol deletion with reference checking |

### Memory System (5)
| Tool | Status | Avg Time | Description |
|------|--------|----------|-------------|
| **`write_memory`** | ✅ | ~0.1s | Write persistent project knowledge |
| **`read_memory`** | ✅ | ~0.1s | Read a memory file |
| **`list_memories`** | ✅ | ~0.1s | List all available memories |
| **`edit_memory`** | ✅ | ~0.1s | Edit memory content via regex |
| **`delete_memory`** | ✅ | ~0.1s | Delete a memory file |

### Project Onboarding (2)
| Tool | Status | Avg Time | Description |
|------|--------|----------|-------------|
| **`onboarding`** | ✅ | ~0.1s | Project structure discovery |
| **`initial_instructions`** | ✅ | ~0.1s | Serena usage instructions |

### ⚠️ Tools Removed from MCP Interface
The following tools exist in the Serena CLI but **are NOT available** through the MCP interface in v1.5.3:
- `list_dir` — use `Read`/`Glob`/`Grep` tools instead
- `read_file` — use `Read` tool instead
- `find_file` — use `Glob` tool instead
- `create_text_file` — use `Write` tool instead
- `execute_shell_command` — use `Bash` tool instead
- `search_for_pattern` — use `Grep` tool instead
- `activate_project` — project is auto-detected from CWD
- `get_current_config` — use `initial_instructions` instead
- `check_onboarding_performed` — use `onboarding` instead

## Validated Usage Patterns

### Pattern 1: Starting Any Task (Fastest — 0.2s)
**Always start here.** Memories are instant and provide critical context.

```
1. list_memories()
   → ["architecture", "build_test_guide", "coding_standards", "project_overview"]

2. read_memory(memory_name="architecture")
   → Clean Architecture rules, dependency flow, crate boundaries
```

**Measured time: 0.21s total**

### Pattern 2: Understanding a New File (~7s after LSP warmup)
```
1. get_symbols_overview(relative_path="crates/mcb-domain/src/lib.rs")
   → {"Module": ["macros", "entities", "error", "events", ...]}

2. find_symbol(name_path_pattern="EmbeddingProvider")
   → Exact file path and line numbers

3. find_implementations(
       name_path="EmbeddingProvider",
       relative_path=".../embedding.rs")
   → All concrete implementations (Ollama, Gemini, FastEmbed, ...)

4. find_referencing_symbols(
       name_path="EmbeddingProvider",
       relative_path=".../embedding.rs",
       max_answer_chars=3000)
   → All usage sites (limited to avoid truncation)
```

**Measured time: ~28s on cold LSP, ~7s after warmup**

### Pattern 3: Before Editing
```
1. get_symbols_overview(relative_path="target_file.rs")
   → Understand structure

2. find_referencing_symbols(name_path="SymbolToEdit", ...)
   → Understand all callers before modifying

3. get_diagnostics_for_file(relative_path="target_file.rs")
   → Check for existing errors
```

### Pattern 4: Making Edits
```
1. replace_content(
       relative_path="...",
       needle="old_text",
       repl="new_text",
       mode="literal")
   → Simple text replacement

2. insert_after_symbol(
       relative_path="...",
       name_path="existing_fn",
       body="\n    pub fn new_fn() {}\n")
   → Add new method after existing one

3. rename_symbol(
       relative_path="...",
       name_path="OldName",
       new_name="NewName")
   → LSP-powered rename (affects all references)
```

## Important Tool Behaviors & Limitations

### `find_declaration` — Regex Capture Group Required
The regex parameter **must contain exactly one capture group** `(pattern)`:

```python
# ❌ WRONG — no capture group
find_declaration(relative_path="...", regex="EmbeddingProvider")

# ✅ CORRECT — one capture group
find_declaration(relative_path="...", regex=r"(EmbeddingProvider)")
```

### `find_referencing_symbols` — Large Results
For widely-used symbols, results can exceed 10,000 characters. **Always set `max_answer_chars`**:

```python
find_referencing_symbols(
    name_path="EmbeddingProvider",
    relative_path="...",
    max_answer_chars=3000  # Limit output
)
```

### `rename_symbol` — Requires Indexed File
`rename_symbol` only works on files already indexed by rust-analyzer. It will fail on newly created files. Run `serena project index` after creating new files.

### `safe_delete_symbol` — Parameter Name
Uses `name_path_pattern` (not `name_path`):

```python
# ✅ CORRECT
safe_delete_symbol(
    relative_path="...",
    name_path_pattern="function_name"
)
```

### `get_symbols_overview` — LSP Warmup
The first call after server start takes ~24s because rust-analyzer initializes. Subsequent calls are fast (~2s). Keep the server alive for multiple operations.

### Memory Tools — Instant
All memory operations (`list_memories`, `read_memory`, `write_memory`, `edit_memory`, `delete_memory`) complete in **under 0.1s**. Use them liberally for context.

## Health Check

Run the comprehensive health check:

```bash
cd /home/marlonsc/mcb
serena project health-check
```

This validates:
- Language server startup
- Symbol overview retrieval
- Symbol finding
- Reference finding
- Pattern search

## Reindexing

After significant code changes or creating new files:

```bash
cd /home/marlonsc/mcb
serena project index --log-level INFO --timeout 60
```

**Required before** using `rename_symbol` or `safe_delete_symbol` on new files.

## Client Configuration

### VS Code
Configured in `~/.config/Code/User/mcp.json`:
```json
"oraios/serena": {
    "type": "stdio",
    "command": "uvx",
    "args": [
        "--from", "git+https://github.com/oraios/serena",
        "serena", "start-mcp-server",
        "--context", "claude-code",
        "--project-from-cwd"
    ]
}
```

### Cursor
Configured in `~/.config/cursor-mcp.json` (same pattern as VS Code).

### OpenCode
Configured in `~/.config/opencode/opencode.json`:
```json
"serena": {
    "type": "local",
    "command": [
        "uvx", "--from", "git+https://github.com/oraios/serena",
        "serena", "start-mcp-server",
        "--context", "claude-code",
        "--project-from-cwd",
        "--mode", "interactive",
        "--mode", "editing"
    ]
}
```

### Kimi Code / Claude
Tools permitted in `.claude/settings.local.json` (20 tools):
- `mcp__serena__get_symbols_overview`
- `mcp__serena__find_symbol`
- `mcp__serena__find_declaration`
- `mcp__serena__find_implementations`
- `mcp__serena__find_referencing_symbols`
- `mcp__serena__get_diagnostics_for_file`
- `mcp__serena__replace_content`
- `mcp__serena__replace_symbol_body`
- `mcp__serena__insert_after_symbol`
- `mcp__serena__insert_before_symbol`
- `mcp__serena__rename_symbol`
- `mcp__serena__safe_delete_symbol`
- `mcp__serena__read_memory`
- `mcp__serena__write_memory`
- `mcp__serena__list_memories`
- `mcp__serena__edit_memory`
- `mcp__serena__delete_memory`
- `mcp__serena__rename_memory`
- `mcp__serena__onboarding`
- `mcp__serena__initial_instructions`

## Memories

Project memories are stored in `.serena/memories/`:

| Memory | Purpose |
|--------|---------|
| `project_overview.md` | Project identity, tech stack, structure |
| `architecture.md` | Clean Architecture rules, DI pattern |
| `coding_standards.md` | Rust conventions, lint policy |
| `build_test_guide.md` | Make commands, quality gates |

## Multi-Session Optimization

> ⚠️ **Warning**: Running multiple agent sessions simultaneously on this machine (62GB RAM, 20 cores) can exhaust available memory. Each session spawns its own `rust-analyzer` instance (~4–8GB RAM) and may trigger parallel cargo builds.

### Problem
Each `serena start-mcp-server` instance launches an independent `rust-analyzer` via LSP. The MCB workspace contains 7 first-party crates plus `third-party/` patches (sea-orm, sea-query, loco, etc.), all of which rust-analyzer analyzes. With 2+ concurrent sessions, RAM usage quickly exceeds 50GB + swap.

### Optimizations Applied

The following project-level optimizations are already configured:

| File | Optimization | Effect |
|------|--------------|--------|
| `.cargo/config.toml` | `jobs = 8` (was `-1`) | Limits cargo parallelism to 8 cores |
| `.cargo/config.toml` | `[env]` `RAYON_NUM_THREADS=4` | Limits rust-analyzer internal threads |
| `Cargo.toml` | `split-debuginfo = "packed"` | Reduces linker memory usage |
| `Cargo.toml` | `build-override.opt-level = 1` (was `3`) | Lower memory for proc-macro/build-script compilation |
| `.vscode/settings.json` | `cachePriming.enable = false` | Disables rust-analyzer warm-up cache |
| `.vscode/settings.json` | `procMacro.enable = false` | Disables proc-macro expansion (large RAM save) |
| `.vscode/settings.json` | `checkOnSave = false` | Disables background `cargo check` |
| `.vscode/settings.json` | `diagnostics.enable = false` | Disables continuous diagnostic analysis |

### Cleanup Script

Run the optimizer before starting a new session:

```bash
# Dry-run (safe — reports only)
make dev-env-optimize

# Actually kill duplicate processes
make dev-env-optimize APPLY=Y
```

This script:
- Detects and kills duplicate `rust-analyzer` instances (keeps 1 most recent)
- Detects and kills duplicate Serena MCP servers (keeps 2 most recent)
- Flags stale `cargo` processes running > 30 minutes
- Prints a resource usage report

### Recommended Workflow

1. **Before starting a new session**, run `make dev-env-optimize APPLY=Y`
2. **Limit concurrent sessions** to 2–3 maximum on this machine
3. **Keep one "primary" session** alive for continuity; kill idle ones
4. **Run builds sequentially** when possible — avoid `cargo check` in 2+ sessions simultaneously

### Environment Variables

These are set automatically via `.cargo/config.toml [env]`:

```bash
export CARGO_BUILD_JOBS=8      # cargo parallelism
export RAYON_NUM_THREADS=4     # rust-analyzer / rayon threads
export RA_LOG=error            # rust-analyzer log level
```

### sccache (Mandatory Compilation Cache)

sccache is **mandatory** for all builds (local and CI). It is configured automatically:

- **Local**: `Makefile` sets `RUSTC_WRAPPER=sccache` unconditionally
- **CI**: `.github/workflows/ci.yml` configures `SCCACHE_GHA_ENABLED=true`

sccache eliminates redundant rebuilds across sessions and CI runs. It is mutually exclusive with Cargo incremental compilation (`CARGO_INCREMENTAL=0`).

To check sccache status:

```bash
sccache --show-stats
```

To reset local cache:

```bash
sccache --zero-stats && sccache --stop-server
```

## Troubleshooting

### Language server timeout
Increase `symbol_info_budget` in `.serena/project.yml` (default: 15s).

### First call is very slow (~24s)
Normal behavior — rust-analyzer initializes on first LSP request. Subsequent calls are fast. Keep the MCP server alive for batch operations.

### `rename_symbol` fails on new files
New files must be indexed first. Run `serena project index` before using `rename_symbol` on recently created files.

### Large search results from `find_referencing_symbols`
Always set `max_answer_chars` parameter (default: 5000, max: 8000).

### Cache issues
Delete `.serena/cache/rust/` and reindex with `serena project index`.

### rust-analyzer warnings
Warnings about "overly long loop turn" during indexing are normal for large workspaces and do not affect functionality.

### Context deprecated warning
If you see `Context name 'ide-assistant' is deprecated and has been renamed to 'claude-code'`, update all client configurations to use `--context claude-code`.

## Global Configuration

The global Serena configuration is in `~/.serena/serena_config.yml`:
- `tool_timeout`: 60s
- `default_max_tool_answer_chars`: 8000
- `symbol_info_budget`: 15.0s
- `token_count_estimator`: TIKTOKEN_GPT4O
- `log_level`: WARNING (30)
