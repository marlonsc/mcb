<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# .MCP-context.toml Schema Documentation

**Version**: 0.2.0

Complete schema reference for `.mcp-context.toml` git-aware indexing configuration.

## Table of Contents

1. [Overview](#overview)
2. [File Location](#file-location)
3. [Schema Reference](#schema-reference)
4. [Examples](#examples)
5. [Pattern Syntax](#pattern-syntax)
6. [Environment Variables](#environment-variables)
7. [Validation](#validation)

---

## Overview

`.mcp-context.toml` is an**optional** configuration file for v0.2.0 that enables:

- Multi-branch indexing with configurable depth
- Smart file filtering with glob patterns
- Submodule support
- Branch-specific configuration

If the file doesn't exist, MCB uses sensible defaults for backward compatibility.

---

## File Location

Place `.mcp-context.toml` in your repository root:

```text
/path/to/repo/
├── .mcp-context.toml    ← Create this file
├── src/
├── Cargo.toml
└── ...
```

---

## Schema Reference

### [git] Section

```toml
[git]

# Depth: Number of commits to analyze

# Type: integer

# Default: 1000

# Valid Range: 1-10000
depth = 100

# Branches: Which branches to index

# Type: array of strings

# Default: ["main", "HEAD"]

# Supports: branch names, patterns (main, develop, feature/*)
branches = ["main", "develop", "staging"]

# Include Submodules: Whether to index submodules

# Type: boolean

# Default: true
include_submodules = true

# Ignore Patterns: Files/directories to skip

# Type: array of strings (1)

# Default: [] (index everything)

# Syntax: glob patterns (see Pattern Syntax section)
ignore_patterns = [
  "target/",
  "*.log",
  "node_modules/",
  ".git/"
]
```

---

## Examples

### Minimal Configuration

```toml
[git]
depth = 100
```

### Typical Rust Project

```toml
[git]
depth = 100
branches = ["main"]
ignore_patterns = [
  "target/",
  "*.log",
  ".git/",
  "Cargo.lock",
  ".vscode/",
  ".idea/"
]
```

### JavaScript/Node Project

```toml
[git]
depth = 50
branches = ["main", "develop"]
ignore_patterns = [
  "node_modules/",
  "dist/",
  "build/",
  "coverage/",
  ".env",
  ".env.local",
  "*.log",
  ".next/",
  "out/"
]
```

### Python Project

```toml
[git]
depth = 75
branches = ["main", "develop"]
include_submodules = true
ignore_patterns = [
  "__pycache__/",
  ".venv/",
  "venv/",
  ".pytest_cache/",
  ".mypy_cache/",
  "*.egg-info/",
  "dist/",
  "build/",
  ".coverage",
  "htmlcov/"
]
```

---

## Pattern Syntax

### Glob Patterns

Patterns follow glob syntax with three types:

#### 1. Directory Patterns (trailing slash)

```toml
ignore_patterns = [
  "target/",      # Ignore 'target' directory anywhere
  "node_modules/",# Ignore 'node_modules' anywhere
  ".git/"         # Ignore .git (usually auto-ignored)
]
```

#### 2. Wildcard Patterns (asterisk prefix)

```toml
ignore_patterns = [
  "*.log",        # All .log files
  "*.tmp",        # All temporary files
  "*.swp",        # Editor swap files
  "*.bak"         # Backup files
]
```

#### 3. Exact/Substring Matches

```toml
ignore_patterns = [
  "Cargo.lock",   # Exact filename
  ".env",         # Exact filename
  "README.bak"    # Exact filename
]
```

### Matching Behavior

- **Directories**: `target/` matches any directory named "target" at any level
- **Extensions**: `*.log` matches any file ending in ".log"
- **Names**: `Cargo.lock` matches exactly that filename

---

## Environment Variables

Override config file values with environment variables (highest precedence):

```bash

# Override depth
export MCP__GIT__DEPTH=50

# Override branches (comma-separated)
export MCP__GIT__BRANCHES="main,develop,feature/*"

# Override include_submodules
export MCP__GIT__INCLUDE_SUBMODULES=false

# Override ignore patterns (comma-separated)
export MCP__GIT__IGNORE_PATTERNS="target/,*.log,node_modules/"
```

---

## Validation

### Valid Configuration

```toml
[git]
depth = 100                           # ✅ Valid (1-10000)
branches = ["main", "develop"]        # ✅ Valid array
include_submodules = true             # ✅ Valid boolean
ignore_patterns = ["*.log", "target/"]# ✅ Valid array
```

### Invalid Configuration

```toml
[git]
depth = 20000    # ❌ Out of range (max 10000)
branches = "main"# ❌ Should be array
include_submodules = "yes"# ❌ Should be boolean
```

### Defaults Used When Missing

```toml

# If .mcp-context.toml is absent or section incomplete
[git]
depth = 1000                    # Default
branches = ["main", "HEAD"]     # Default
include_submodules = true       # Default
ignore_patterns = []            # Default (no ignores)
```

---

## Complete Example

```toml

# .mcp-context.toml

# MCB v0.2.0 - Git-aware indexing configuration

[git]

# How many commits to analyze

# Use 50-100 for quick indexing, 500+ for comprehensive analysis
depth = 100

# Which branches to index for analysis

# Supports branch names or patterns
branches = [
  "main",
  "develop",
  "feature/*"
]

# Include git submodules in the index
include_submodules = true

# Files and directories to ignore

# Supports: directories (dir/), extensions (*.ext), exact names (file.txt)
ignore_patterns = [
  # Build artifacts
  "target/",
  "dist/",
  "build/",
  "out/",

  ## Dependencies
  "node_modules/",
  ".venv/",
  "venv/",
  "__pycache__/",

  # IDE/Editor
  ".vscode/",
  ".idea/",
  ".vs/",
  "*.swp",
  "*.swo",

  # Lock files
  "Cargo.lock",
  "package-lock.json",
  "yarn.lock",
  "poetry.lock",

  # Logs and temp
  "*.log",
  "*.tmp",
  "*.bak",

  # System
  ".DS_Store",
  "Thumbs.db",

  # Git
  ".git/",
  ".gitignore"
]
```

---

## Migration from v0.1.x

In v0.1.x, all indexing used fixed defaults. v0.2.0 allows customization:

| Feature | v0.1.x | v0.2.0 |
| --------- | -------- | -------- |
| Depth | Hardcoded | Configurable via `.mcp-context.toml` |
| Branches | All branches | Select specific branches |
| Patterns | None | Glob patterns supported |
| Config file | None | `.mcp-context.toml` (optional) |

**Backward Compatible**: If `.mcp-context.toml` is missing, v0.2.0 behaves like v0.1.x.

---

**Last Updated**: 2026-02-05
**Version**: 0.2.0
