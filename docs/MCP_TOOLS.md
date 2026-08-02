<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 MD024 -->
# MCP Tools Schema Documentation

**Version**: 0.3.1
**Last Updated**: 2026-06-04

MCB exposes 24 public tool names through the MCP protocol. `tools/list` returns
the single-purpose names below; implementation routes them through 9 handler
families.

| Family | Tool names returned by `tools/list` |
| ------ | ----------------------------------- |
| Search | `search_code`, `search_memory` |
| Index | `index_repo`, `index_status`, `clear_index` |
| Validate | `validate_code`, `analyze_code`, `list_rules` |
| Memory | `store_memory`, `get_memories`, `list_memories`, `memory_timeline`, `inject_context` |
| Session | `start_session`, `get_session`, `list_sessions`, `summarize_session` |
| Agent | `log_tool_call`, `log_delegation` |
| VCS | `list_repos`, `compare_branches`, `analyze_impact` |
| Project | `project` |
| Entity | `entity` |

The sections below document the shared handler-family schemas used by the
single-purpose tools.

---

## 1. Index Tool Family

Index operations (start, git_index, status, clear).

**Actions**: `start`, `git_index`, `status`, `clear`

| Parameter | Type | Required | Description |
| ----------- | ------ | ---------- | ------------- |
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

## 2. Search Tool Family

Search operations for code and memory.

**Resources**: `code`, `memory`, `context`

| Parameter | Type | Required | Description |
| ----------- | ------ | ---------- | ------------- |
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

## 3. Validate Tool Family

Validation and analysis operations.

**Actions**: `run`, `list_rules`, `analyze`

| Parameter | Type | Required | Description |
| ----------- | ------ | ---------- | ------------- |
| `action` | enum | **yes** | `run`, `list_rules`, `analyze` |
| `scope` | enum | no | `file` or `project` |
| `path` | string | no | Path to file or project directory |
| `rules` | string[] | no | Specific rules to run (empty = all) |
| `category` | string | no | Rule category filter |

---

## 4. Memory Tool Family

Memory storage, retrieval, and timeline operations.

**Actions**: `store`, `get`, `list`, `timeline`, `inject`

**Resources**: `observation`, `execution`, `quality_gate`, `error_pattern`, `session`

| Parameter | Type | Required | Description |
| ----------- | ------ | ---------- | ------------- |
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

## 5. Session Tool Family

Session lifecycle operations.

**Actions**: `create`, `get`, `update`, `list`, `summarize`

| Parameter | Type | Required | Description |
| ----------- | ------ | ---------- | ------------- |
| `action` | enum | **yes** | `create`, `get`, `update`, `list`, `summarize` |
| `session_id` | string | no | Session ID (required for get/update/summarize) |
| `data` | object | no | Data payload for create/update |
| `project_id` | string | no | Filter by project ID |
| `worktree_id` | string | no | Filter by worktree ID |
| `agent_type` | string | no | Filter by agent type |
| `status` | string | no | Filter by status |
| `limit` | integer | no | Maximum results for list |

---

## 6. Agent Tool Family

Agent activity logging operations.

**Actions**: `log_tool`, `log_delegation`

| Parameter | Type | Required | Description |
| ----------- | ------ | ---------- | ------------- |
| `action` | enum | **yes** | `log_tool`, `log_delegation` |
| `session_id` | string | **yes** | Session ID for the agent |
| `data` | object | **yes** | Activity data payload |

---

## 7. `project` Tool

Project workflow management (phases, issues, dependencies, decisions).

**Actions**: `create`, `get`, `update`, `list`, `delete`

**Resources**: `project`, `phase`, `issue`, `dependency`, `decision`

| Parameter | Type | Required | Description |
| ----------- | ------ | ---------- | ------------- |
| `action` | enum | **yes** | `create`, `get`, `update`, `list`, `delete` |
| `resource` | enum | **yes** | `project`, `phase`, `issue`, `dependency`, `decision` |
| `project_id` | string | **yes** | Project ID |
| `data` | object | no | Data payload for create/update |
| `filters` | object | no | Additional filters for list |

---

## 8. VCS Tool Family

Version control operations (list, compare, impact).

**Public tools**: `list_repos`, `compare_branches`, `analyze_impact`

| Parameter | Type | Required | Description |
| ----------- | ------ | ---------- | ------------- |
| `action` | enum | internal | `list_repositories`, `compare_branches`, `analyze_impact` |
| `repo_id` | string | no | Repository identifier, usually injected by context |
| `repo_path` | string | no | Repository path on disk, usually injected by context |
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
| ----------- | ------ | ---------- | ------------- |
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
| ------- | ------ | ---------- |
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

## Operation Family Mode Matrix

| Tool | `stdio-only` | `client-hybrid` | `server-hybrid` |
| ------ |:---:|:---:|:---:|
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
