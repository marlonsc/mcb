# MCP Tools Schema Documentation

**Version**: 0.2.0  
**Last Updated**: 2026-02-05

This document provides complete schema documentation for all 8 MCP tools available in MCB v0.2.0.

---

## 1. INDEX_CODEBASE Tool

**Purpose**: Index a codebase for semantic search

**Parameters**:

```json
{
  "collection": {
    "type": "string",
    "required": true,
    "description": "Collection name (alphanumeric, underscores, hyphens)"
  },
  "repository_path": {
    "type": "string",
    "required": true,
    "description": "Path to git repository or directory to index"
  },
  "depth": {
    "type": "integer",
    "required": false,
    "default": 1000,
    "description": "Number of commits to analyze (git repositories)"
  }
}
```

**Return Type**:

```json
{
  "status": "string (started|completed|failed)",
  "collection": "string",
  "files_processed": "integer",
  "chunks_created": "integer",
  "operation_id": "string (UUID)",
  "error": "string (if failed)"
}
```

**Example Usage**:

```bash
mcb index_codebase \
  --collection my_project \
  --repository_path /path/to/repo \
  --depth 100
```

---

## 2. SEARCH_CODE Tool

**Purpose**: Search indexed codebase with semantic queries

**Parameters**:

```json
{
  "query": {
    "type": "string",
    "required": true,
    "description": "Natural language search query"
  },
  "collection": {
    "type": "string",
    "required": true,
    "description": "Collection to search in"
  },
  "limit": {
    "type": "integer",
    "required": false,
    "default": 10,
    "minimum": 1,
    "maximum": 100,
    "description": "Maximum results to return"
  },
  "filter": {
    "type": "object",
    "required": false,
    "properties": {
      "languages": { "type": "array of strings" },
      "file_extensions": { "type": "array of strings" }
    }
  }
}
```

**Return Type**:

```json
{
  "results": [
    {
      "file": "string",
      "content": "string (code snippet)",
      "relevance_score": "float (0-1)",
      "line_number": "integer"
    }
  ],
  "count": "integer",
  "query_time_ms": "integer"
}
```

**Example Usage**:

```bash
mcb search_code \
  --query "authenticate user" \
  --collection my_project \
  --limit 5
```

---

## 3. GET_INDEXING_STATUS Tool

**Purpose**: Get current indexing status for a collection

**Parameters**:

```json
{
  "collection": {
    "type": "string",
    "required": true,
    "description": "Collection name"
  }
}
```

**Return Type**:

```json
{
  "is_indexing": "boolean",
  "progress_percent": "float (0-100)",
  "files_processed": "integer",
  "files_total": "integer",
  "current_file": "string",
  "estimated_time_remaining": "integer (seconds)"
}
```

---

## 4. CLEAR_INDEX Tool

**Purpose**: Clear indexed data from a collection

**Parameters**:

```json
{
  "collection": {
    "type": "string",
    "required": true,
    "description": "Collection to clear"
  }
}
```

**Return Type**:

```json
{
  "status": "string (cleared|failed)",
  "collection": "string",
  "items_deleted": "integer"
}
```

---

## 5. MEMORY Tool (NEW in v0.2.0)

**Purpose**: Store and retrieve session observations

**Sub-operations**:

-   `store`: Save observation
-   `search`: Find observations
-   `timeline`: Get temporal sequence
-   `get`: Retrieve specific observations
-   `inject`: Get context for session

**Parameters** (vary by operation):

### 5.1 Store Observation

```json
{
  "operation": "store",
  "content": {
    "type": "string",
    "required": true,
    "description": "Observation content"
  },
  "observation_type": {
    "type": "string",
    "enum": ["Decision", "Execution", "Summary", "Insight", "Pattern"],
    "required": true
  },
  "tags": {
    "type": "array of strings",
    "required": false,
    "description": "Categorization tags"
  }
}
```

### 5.2 Search Memory

```json
{
  "operation": "search",
  "query": {
    "type": "string",
    "required": true,
    "description": "Search query"
  },
  "tags": {
    "type": "array of strings",
    "required": false,
    "description": "Filter by tags"
  },
  "limit": {
    "type": "integer",
    "required": false,
    "default": 10
  }
}
```

**Return Type**:

```json
{
  "observations": [
    {
      "id": "string (UUID)",
      "content": "string",
      "type": "string",
      "relevance_score": "float",
      "created_at": "ISO8601 timestamp",
      "tags": "array of strings"
    }
  ]
}
```

---

## 6. SESSION Tool (NEW in v0.2.0)

**Purpose**: Manage session context and memory

**Parameters**:

```json
{
  "operation": {
    "type": "string",
    "enum": ["start", "end", "get_context"],
    "required": true
  },
  "session_id": {
    "type": "string",
    "required": true,
    "description": "Session identifier"
  }
}
```

**Return Type**:

```json
{
  "session_id": "string",
  "status": "string (started|ended|active)",
  "context": "object (if get_context)",
  "timestamp": "ISO8601"
}
```

---

## 7. VCS Tool (NEW in v0.2.0)

**Purpose**: Version control operations (git-aware indexing)

**Sub-operations**:

-   `index_repository`: Index with git metadata
-   `search_branch`: Search within branch
-   `compare_branches`: Compare branches
-   `analyze_impact`: Analyze commit impact

### 7.1 Index Repository

```json
{
  "operation": "index_repository",
  "collection": "string",
  "repository_path": "string",
  "branches": ["main", "develop"],
  "depth": 100,
  "ignore_patterns": ["*.log", "target/", "node_modules"]
}
```

### 7.2 Compare Branches

```json
{
  "operation": "compare_branches",
  "collection": "string",
  "branch_a": "main",
  "branch_b": "develop"
}
```

**Return Type**:

```json
{
  "branch_a": "string",
  "branch_b": "string",
  "commits_ahead": "integer",
  "commits_behind": "integer",
  "files_changed": "integer",
  "differences": "array of objects"
}
```

---

## 8. AGENT Tool (v0.2.0)

**Purpose**: Manage agent sessions and tracking

**Parameters**:

```json
{
  "operation": {
    "type": "string",
    "enum": ["create_session", "track_execution", "get_state"],
    "required": true
  },
  "agent_name": {
    "type": "string",
    "required": true
  }
}
```

**Return Type**:

```json
{
  "agent_id": "string (UUID)",
  "status": "string",
  "session_active": "boolean"
}
```

---

## 9. PROJECT Tool (v0.2.0)

**Purpose**: Project-level operations

**Parameters**:

```json
{
  "operation": {
    "type": "string",
    "enum": ["list", "get_stats", "get_health"],
    "required": true
  }
}
```

---

## Error Response Format

All tools return errors in consistent format:

```json
{
  "error": {
    "type": "string (InvalidInput|NotFound|InternalError|Unavailable)",
    "message": "string",
    "details": "string (optional)"
  }
}
```

---

## v0.2.0 Changes from v0.2.0

**New Tools**:

-   MEMORY (observation storage + search)
-   SESSION (session context management)
-   VCS (git-aware indexing)
-   AGENT (agent session tracking)

**Enhanced Tools**:

-   INDEX_CODEBASE: Added `depth` parameter + git support
-   SEARCH_CODE: Added filtering support

**Deprecated**: None

---

## Configuration

See [CONFIGURATION.md](./CONFIGURATION.md) for `.mcp-context.toml` setup to customize tool behavior.
