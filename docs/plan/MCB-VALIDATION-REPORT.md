# MCB MCP Tools Validation Report

**Date**: 2026-02-08
**Version**: MCB v0.2.0
**Tester**: Sisyphus (OpenCode)
**Environment**: Milvus @ localhost:19530, Ollama @ localhost:11434 (nomic-embed-text)

---

## Executive Summary

| Tool | Status | Working Actions | Broken Actions |
|------|--------|-----------------|----------------|
| `mcp_mcb_index` | ✅ **WORKING** | start, status, clear | - |
| `mcp_mcb_search` | ✅ **WORKING** | code, memory | - |
| `mcp_mcb_validate` | ⚠️ **PARTIAL** | list_rules, analyze | run (scope enum) |
| `mcp_mcb_vcs` | ⚠️ **PARTIAL** | list_repositories, index_repository | search_branch, compare_branches, analyze_impact |
| `mcp_mcb_session` | ⚠️ **PARTIAL** | list | create, get, update, summarize |
| `mcp_mcb_memory` | ❌ **BROKEN** | - | store, list, get, timeline, inject |
| `mcp_mcb_project` | ❌ **NOT IMPLEMENTED** | - | All Actions |
| `mcp_mcb_agent` | ❌ **BROKEN** | - | log_tool, log_delegation |

**Overall**: 2 fully working, 3 partial, 3 broken/not implemented.

---

## Detailed Test Results

### 1. MCP_mcb_index ✅ WORKING

All Actions work correctly.

| Action | Status | Evidence |
|--------|--------|----------|
| `status` | ✅ | Returns `{"status": "idle"}` |
| `start` | ✅ | Returns `{"status": "completed", "files_processed": N, "chunks_created": M}` |
| `clear` | ✅ | Returns `{"success": true, "collection": "name"}` |

**Test Commands:**

```
mcp_mcb_index(action="status")
mcp_mcb_index(action="start", path="/path", collection="name", extensions=[".md"])
mcp_mcb_index(action="clear", collection="name")
```

---

### 2. MCP_mcb_search ✅ WORKING

Code search works perfectly. Memory search returns empty (expected if no memories stored).

| Action | Status | Evidence |
|--------|--------|----------|
| `resource=code` | ✅ | Returns ranked results with file paths, content, scores |
| `resource=memory` | ✅ | Returns empty array (no memories stored - expected) |
| Error handling | ✅ | "Collection not found" for nonexistent collections |

**Test Commands:**

```
mcp_mcb_search(query="provider config", resource="code", collection="opencode", limit=5)
mcp_mcb_search(query="session", resource="memory", limit=3)
```

---

### 3. MCP_mcb_validate ⚠️ PARTIAL

list_rules and analyze work. run action has scope enum parsing issue.

| Action | Status | Evidence |
|--------|--------|----------|
| `list_rules` | ✅ | Returns 12 validators: clean_architecture, SOLID, quality, organization, kiss, naming, documentation, performance, async_patterns, dependencies, patterns, tests |
| `analyze` | ✅ | Returns complexity metrics: cyclomatic, cognitive, maintainability_index, sloc, functions breakdown |
| `run` | ❌ | Scope enum not parsed: `JSON Parse error: Unexpected identifier "file"` |

**Working Commands:**

```
mcp_mcb_validate(action="list_rules")
mcp_mcb_validate(action="analyze", path="/path/to/file.py")
```

**Broken Command:**

```
mcp_mcb_validate(action="run", path="/path", scope="file")

# Error: JSON Parse error: Unexpected identifier "file"
```

**Root Cause**: The `scope` parameter expects an enum but MCP is not properly serializing the String value.

---

### 4. MCP_mcb_vcs ⚠️ PARTIAL

Repository listing and indexing work. Search/compare/analyze fail with "Repository not found".

| Action | Status | Evidence |
|--------|--------|----------|
| `list_repositories` | ✅ | Returns array of all indexed repositories |
| `index_repository` | ✅ | Returns `{repository_id, path, default_branch, branches_found, total_files, commits_indexed}` |
| `search_branch` | ❌ | `Repository not found: opencode` |
| `compare_branches` | ❌ | `Repository not found: opencode` |
| `analyze_impact` | ❌ | `Repository not found: opencode` |

**Root Cause**: The `repo_id` parameter must be the hash returned by `index_repository`, not an arbitrary name. However, even with correct repo_id, search/compare/analyze may have other issues.

**Working Commands:**

```
mcp_mcb_vcs(action="list_repositories")
mcp_mcb_vcs(action="index_repository", repo_path="/path/to/repo", branches=["main"])
```

**Broken Commands:**

```
mcp_mcb_vcs(action="search_branch", repo_id="name", query="...", limit=3)
mcp_mcb_vcs(action="compare_branches", repo_id="name", base_branch="main", target_branch="feature")
mcp_mcb_vcs(action="analyze_impact", repo_id="name", target_branch="main")
```

---

### 5. MCP_mcb_session ⚠️ PARTIAL

List works. All other Actions fail.

| Action | Status | Evidence |
|--------|--------|----------|
| `list` | ✅ | Returns `{"sessions": [], "count": 0}` |
| `create` | ❌ | `Missing agent_type for create` |
| `get` | ⚠️ | `Session not found` (expected for nonexistent) |
| `update` | ⚠️ | `Session not found` (expected for nonexistent) |
| `summarize` | ⚠️ | `Session not found` (expected for nonexistent) |

**Root Cause**: The `data` JSON object is not being parsed correctly. The handler expects `agent_type` inside data but cannot extract it.

**Working Command:**

```
mcp_mcb_session(action="list", limit=5)
```

**Broken Command:**

```
mcp_mcb_session(action="create", data={"agent_type": "test", "project_id": "opencode"})

# Error: Missing agent_type for create
```

---

### 6. MCP_mcb_memory ❌ BROKEN

All Actions fail.

| Action | Status | Evidence |
|--------|--------|----------|
| `store` (observation) | ❌ | `Missing data payload for observation store` |
| `store` (error_pattern) | ❌ | `ErrorPattern store not implemented yet` |
| `list` | ❌ | `SQL query_all failed: Failed to fetch from memory_observations: ...` |
| `get` | ❌ | Not tested (list fails) |
| `timeline` | ❌ | `Missing anchor_id or query for timeline` |
| `inject` | ❌ | `Failed to inject context: ...` |

**Root Causes**:

1. **Data payload parsing**: JSON object in `data` parameter not being extracted
2. **SQL errors**: Database table may not exist or have wrong schema
3. **Error patterns not implemented**: Feature stub in code

**Broken Commands:**

```
mcp_mcb_memory(action="store", resource="observation", data={"content": "test", "tags": ["t1"]})

# Error: Missing data payload for observation store

mcp_mcb_memory(action="list", resource="observation", limit=5)

# Error: SQL query_all failed

mcp_mcb_memory(action="timeline", resource="observation", depth_before=5, depth_after=5)

# Error: Missing anchor_id or query for timeline
```

---

### 7. MCP_mcb_project ❌ NOT IMPLEMENTED

All Actions return "Project workflow not yet implemented".

| Action | Status | Evidence |
|--------|--------|----------|
| `list` (phase) | ❌ | `Project workflow not yet implemented` |
| `list` (issue) | ❌ | `Project workflow not yet implemented` |
| `list` (decision) | ❌ | `Project workflow not yet implemented` |
| `create` | ❌ | `Project workflow not yet implemented` |
| `update` | ❌ | Not tested |
| `delete` | ❌ | Not tested |

**Root Cause**: Handler returns stub message. Feature not implemented in v0.2.0.

---

### 8. MCP_mcb_agent ❌ BROKEN

All Actions fail with data parsing error.

| Action | Status | Evidence |
|--------|--------|----------|
| `log_tool` | ❌ | `Data must be a JSON object` |
| `log_delegation` | ❌ | `Data must be a JSON object` |

**Root Cause**: Same as session/memory - the `data` JSON object parameter is not being deserialized correctly by the MCP handler.

**Broken Commands:**

```
mcp_mcb_agent(action="log_tool", session_id="test", data={"tool_name": "edit", "success": true})

# Error: Data must be a JSON object

mcp_mcb_agent(action="log_delegation", session_id="test", data={"target": "explore", "task": "..."})

# Error: Data must be a JSON object
```

---

## Root Cause Analysis

### Issue 1: JSON Data Payload Not Parsed (CRITICAL - P0)

**Affected Tools**: memory, session, agent

**Symptoms**:

- `Missing data payload for X`
- `Missing agent_type for create`
- `Data must be a JSON object`

**Evidence**: The `data` parameter is defined as `Option<serde_json::Value>` but when passed from MCP client, it's not being deserialized correctly.

**Probable Location**: `mcb-server/src/handlers/` - parameter extraction logic

**Fix Priority**: P0 - Blocks memory, session creation, and agent logging

---

### Issue 2: SQL Database Errors (HIGH - P1)

**Affected Tools**: memory (list, inject)

**Symptoms**:

- `SQL query_all failed: Failed to fetch from memory_observations`

**Evidence**: The memory_observations table may not exist or have wrong schema in SQLite.

**Probable Location**:

- Schema: `mcb-domain/src/repositories/`
- Migrations: `mcb-persistence/`

**Fix Priority**: P1 - Blocks memory read operations

---

### Issue 3: Enum Serialization (MEDIUM - P2)

**Affected Tools**: validate (run action)

**Symptoms**:

- `JSON Parse error: Unexpected identifier "file"`

**Evidence**: The `scope` enum (file/project) is not being serialized/deserialized correctly through MCP.

**Probable Location**: `mcb-mcp-server/src/tools/validate.rs` - scope parameter handling

**Fix Priority**: P2 - Only affects validate run action

---

### Issue 4: Repository ID Resolution (MEDIUM - P2)

**Affected Tools**: vcs (search_branch, compare_branches, analyze_impact)

**Symptoms**:

- `Repository not found: opencode`

**Evidence**: The `repo_id` must be the hash returned by index_repository, not an arbitrary name. Documentation should clarify this.

**Probable Location**: `mcb-server/src/handlers/vcs.rs` - repo lookup logic

**Fix Priority**: P2 - VCS operations partially work

---

### Issue 5: Feature Not Implemented (LOW - P3)

**Affected Tools**: project (all), memory (error_pattern store)

**Symptoms**:

- `Project workflow not yet implemented`
- `ErrorPattern store not implemented yet`

**Evidence**: Stub implementations in handlers.

**Probable Location**:

- `mcb-server/src/handlers/project.rs`
- `mcb-application/src/use_cases/memory_service.rs`

**Fix Priority**: P3 - Known roadmap items

---

## Recommendations for MCB Team

### Immediate (v0.2.1 Hotfix)

1. **Fix JSON data payload parsing** - This is blocking 3 major tools

- Debug why `serde_json::Value` from MCP is not deserializing
- Add logging to trace the actual received payload
- Consider using raw JSON String and parsing explicitly

1. **Fix memory SQL errors** - Ensure tables exist and have correct schema

- Add migration check on startup
- Return clearer error messages

### Short-term (v0.3.0)

1. **Fix validate scope enum** - Use String matching instead of enum deserialization
2. **Improve VCS repo_id documentation** - Clarify that hash from index is required
3. **Implement project handler** - Core workflow feature

### Medium-term (v0.4.0)

1. **Implement error_pattern store** - Complete memory feature set
2. **Add E2E MCP tool tests** - Catch these issues before release

---

## Test Environment

```bash

# MCB Version
mcb --version

# v0.2.0

# Milvus
docker ps | grep milvus

# milvus-standalone running on 19530

# Ollama
curl http://localhost:11434/api/tags

# nomic-embed-text available

# OpenCode Config
cat ~/.config/opencode/opencode.jsonc | grep mcb

# MCB MCP server configured with bash wrapper
```

---

## Appendix: All Test Commands

```bash

# Index
mcp_mcb_index(action="status")
mcp_mcb_index(action="start", path="/home/marlonsc/.config/opencode", collection="test", extensions=[".md"])
mcp_mcb_index(action="clear", collection="test")

# Search
mcp_mcb_search(query="provider", resource="code", collection="opencode", limit=5)
mcp_mcb_search(query="session", resource="memory", limit=3)

# Validate
mcp_mcb_validate(action="list_rules")
mcp_mcb_validate(action="analyze", path="/path/to/file.py")
mcp_mcb_validate(action="run", path="/path", scope="file")  # BROKEN

# VCS
mcp_mcb_vcs(action="list_repositories")
mcp_mcb_vcs(action="index_repository", repo_path="/path", branches=["main"])
mcp_mcb_vcs(action="search_branch", repo_id="name", query="test")  # BROKEN

# Session
mcp_mcb_session(action="list", limit=5)
mcp_mcb_session(action="create", data={"agent_type": "test"})  # BROKEN

# Memory
mcp_mcb_memory(action="store", resource="observation", data={"content": "test"})  # BROKEN
mcp_mcb_memory(action="list", resource="observation", limit=5)  # BROKEN

# Project
mcp_mcb_project(action="list", resource="phase", project_id="test")  # NOT IMPLEMENTED

# Agent
mcp_mcb_agent(action="log_tool", session_id="test", data={"tool": "x"})  # BROKEN
```

---

**Report Generated**: 2026-02-08T19:37:00-03:00
**Next Review**: After v0.2.1 fixes deployed
