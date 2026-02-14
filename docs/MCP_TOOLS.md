<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 MD024 -->
# MCP Tools Schema Documentation

**Version**: 0.2.1
**Last Updated**: 2026-02-14

MCB exposes 9 tools through the MCP protocol. Tool names as returned by `tools/list`:

| # | Tool | Description |
|---|------|-------------|
| 1 | `index` | Index operations (start, status, clear) |
| 2 | `search` | Search operations for code and memory |
| 3 | `validate` | Validation and analysis operations |
| 4 | `memory` | Memory storage, retrieval, and timeline operations |
| 5 | `session` | Session lifecycle operations |
| 6 | `agent` | Agent activity logging operations |
| 7 | `project` | Project workflow management (phases, issues, dependencies, decisions) |
| 8 | `vcs` | Version control operations (list, index, compare, search, impact) |
| 9 | `entity` | Unified entity CRUD (vcs/plan/issue/org resources) |

---

## 1. `index` Tool

Index operations (start, git_index, status, clear).

**Actions**: `start`, `git_index`, `status`, `clear`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `action` | enum | **yes** | `start`, `git_index`, `status`, `clear` |
| `path` | string | no | Path to codebase directory (required for `start`) |
| `collection` | string | no | Collection name for the index |
| `extensions` | string[] | no | File extensions to include |
| `exclude_dirs` | string[] | no | Directories to exclude |
| `ignore_patterns` | string[] | no | Glob patterns for files/directories to exclude |
| `max_file_size` | integer | no | Maximum file size to index (bytes) |
| `follow_symlinks` | boolean | no | Follow symbolic links during indexing |
| `token` | string | no | JWT token for authenticated requests |

---

## 2. `search` Tool

Search operations for code and memory.

**Resources**: `code`, `memory`, `context`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | **yes** | Natural language search query |
| `resource` | enum | **yes** | `code`, `memory`, `context` |
| `collection` | string | no | Collection name |
| `extensions` | string[] | no | File extensions to include (code search) |
| `filters` | string[] | no | Additional search filters |
| `limit` | integer | no | Maximum results to return |
| `min_score` | float | no | Minimum similarity score (0.0–1.0) |
| `tags` | string[] | no | Filter by tags (memory search) |
| `session_id` | string | no | Filter by session ID (memory search) |
| `token` | string | no | JWT token for authenticated requests |

---

## 3. `validate` Tool

Validation and analysis operations.

**Actions**: `run`, `list_rules`, `analyze`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `action` | enum | **yes** | `run`, `list_rules`, `analyze` |
| `scope` | enum | no | `file` or `project` |
| `path` | string | no | Path to file or project directory |
| `rules` | string[] | no | Specific rules to run (empty = all) |
| `category` | string | no | Rule category filter |

---

## 4. `memory` Tool

Memory storage, retrieval, and timeline operations.

**Actions**: `store`, `get`, `list`, `timeline`, `inject`

**Resources**: `observation`, `execution`, `quality_gate`, `error_pattern`, `session`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `action` | enum | **yes** | `store`, `get`, `list`, `timeline`, `inject` |
| `resource` | enum | **yes** | `observation`, `execution`, `quality_gate`, `error_pattern`, `session` |
| `data` | object | no | Data payload for store action |
| `ids` | string[] | no | Resource IDs for get action |
| `query` | string | no | Query string for list/search |
| `tags` | string[] | no | Filter by tags |
| `session_id` | string | no | Filter by session ID |
| `project_id` | string | no | Filter by project ID |
| `repo_id` | string | no | Filter by repository ID |
| `limit` | integer | no | Maximum results |
| `anchor_id` | string | no | Anchor observation ID (timeline) |
| `depth_before` | integer | no | Timeline depth before anchor |
| `depth_after` | integer | no | Timeline depth after anchor |
| `window_secs` | integer | no | Time window in seconds (timeline) |
| `observation_types` | string[] | no | Observation types to include (inject) |
| `max_tokens` | integer | no | Maximum token budget for injected context |

---

## 5. `session` Tool

Session lifecycle operations.

**Actions**: `create`, `get`, `update`, `list`, `summarize`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `action` | enum | **yes** | `create`, `get`, `update`, `list`, `summarize` |
| `session_id` | string | no | Session ID (required for get/update/summarize) |
| `data` | object | no | Data payload for create/update |
| `project_id` | string | no | Filter by project ID |
| `worktree_id` | string | no | Filter by worktree ID |
| `agent_type` | string | no | Filter by agent type |
| `status` | string | no | Filter by status |
| `limit` | integer | no | Maximum results for list |

---

## 6. `agent` Tool

Agent activity logging operations.

**Actions**: `log_tool`, `log_delegation`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `action` | enum | **yes** | `log_tool`, `log_delegation` |
| `session_id` | string | **yes** | Session ID for the agent |
| `data` | object | **yes** | Activity data payload |

---

## 7. `project` Tool

Project workflow management (phases, issues, dependencies, decisions).

**Actions**: `create`, `get`, `update`, `list`, `delete`

**Resources**: `project`, `phase`, `issue`, `dependency`, `decision`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `action` | enum | **yes** | `create`, `get`, `update`, `list`, `delete` |
| `resource` | enum | **yes** | `project`, `phase`, `issue`, `dependency`, `decision` |
| `project_id` | string | **yes** | Project ID |
| `data` | object | no | Data payload for create/update |
| `filters` | object | no | Additional filters for list |

---

## 8. `vcs` Tool

Version control operations (list, index, compare, search, impact).

**Actions**: `list_repositories`, `index_repository`, `compare_branches`, `search_branch`, `analyze_impact`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `action` | enum | **yes** | `list_repositories`, `index_repository`, `compare_branches`, `search_branch`, `analyze_impact` |
| `repo_id` | string | **yes** | Repository identifier |
| `repo_path` | string | **yes** | Repository path on disk |
| `base_branch` | string | no | Base branch name |
| `target_branch` | string | no | Compare/target branch name |
| `query` | string | no | Search query for branch search |
| `branches` | string[] | no | Branches to index |
| `include_commits` | boolean | no | Include commit history when indexing |
| `depth` | integer | no | Commit history depth |
| `limit` | integer | no | Limit for search or list actions |

---

## 9. `entity` Tool

Unified entity CRUD (vcs/plan/issue/org resources).

**Actions**: `create`, `get`, `update`, `list`, `delete`, `release`

**Resources**: `repository`, `branch`, `worktree`, `assignment`, `plan`, `version`, `review`, `issue`, `comment`, `label`, `label_assignment`, `org`, `user`, `team`, `team_member`, `api_key`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `action` | enum | **yes** | `create`, `get`, `update`, `list`, `delete`, `release` |
| `resource` | enum | **yes** | See resources list above |
| `data` | object | no | JSON payload for create/update |
| `id` | string | no | Resource ID (for get/update/delete/release) |
| `org_id` | string | no | Organization ID |
| `repository_id` | string | no | Repository ID (branch/worktree list) |
| `plan_id` | string | no | Plan ID (version list) |
| `plan_version_id` | string | no | Plan version ID (review list) |
| `issue_id` | string | no | Issue ID (comment/label operations) |
| `label_id` | string | no | Label ID (label unassignment) |
| `project_id` | string | no | Project ID (project-scoped list) |
| `team_id` | string | no | Team ID (team member list) |
| `user_id` | string | no | User ID (team member delete) |
| `worktree_id` | string | no | Worktree ID (assignment list) |
| `email` | string | no | User email (lookup operations) |

---

## Provenance Requirements

Tools `index`, `search`, and `memory` require full execution provenance:

| Field | Type | Required |
|-------|------|----------|
| `session_id` | string | **yes** |
| `project_id` | string | **yes** |
| `repo_id` | string | **yes** |
| `repo_path` | string | **yes** |
| `worktree_id` | string | **yes** |
| `operator_id` | string | **yes** |
| `machine_id` | string | **yes** |
| `agent_program` | string | **yes** |
| `model_id` | string | **yes** |
| `delegated` | boolean | **yes** |
| `timestamp` | integer | **yes** |

When `delegated` is `true`, `parent_session_id` is also required.

---

## Operation Mode Matrix

| Tool | `stdio-only` | `client-hybrid` | `server-hybrid` |
|------|:---:|:---:|:---:|
| `index` | ✅ | ✅ | ✅ |
| `search` | ✅ | ❌ | ✅ |
| `validate` | ✅ | ✅ | ❌ |
| `memory` | ✅ | ❌ | ✅ |
| `session` | ✅ | ❌ | ✅ |
| `agent` | ✅ | ❌ | ✅ |
| `project` | ✅ | ❌ | ✅ |
| `vcs` | ✅ | ❌ | ✅ |
| `entity` | ✅ | ❌ | ✅ |

---

## Error Response Format

All tools return errors via JSON-RPC 2.0 error objects:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Missing execution provenance for 'index': session_id, project_id"
  }
}
```

Error codes follow JSON-RPC 2.0 conventions:
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

Internal error details are sanitized — no stack traces or implementation details leak to clients.

---

## Configuration

See [CONFIGURATION.md](./CONFIGURATION.md) for `.mcp-context.toml` setup to customize tool behavior.
