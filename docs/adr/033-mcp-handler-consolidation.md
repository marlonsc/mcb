---
adr: 33
title: MCP Handler Consolidation
status: ACCEPTED
created:
updated: 2026-02-05
related: []
supersedes: []
superseded_by: []
implementation_status: Incomplete
---

## ADR-033: MCP Handler Consolidation

## Status

Accepted

## Context

The MCP server has grown to 38 tools, creating cognitive overhead for users and maintenance burden. Many tools follow CRUD patterns that can be using parameterization.

### Current Tool Inventory (38 tools)

| Category | Tools | Count |
| ---------- | ------- | ------- |
| Index/Search | index (action=start), search (resource=code), index (action=status), index (action=clear) | 4 |
| Validation | validate (action=run, scope=project), validate (action=run, scope=file), validate (action=list_rules), validate (action=list_rules), validate (action=analyze) | 5 |
| Memory (Legacy) | memory (action=store, resource=observation), search (resource=memory), session (action=summarize), session (action=summarize) | 4 |
| Memory (New) | memory (action=timeline, resource=observation), memory (action=get, resource=observation), memory (action=inject, resource=observation), memory (action=list, resource=observation), memory (action=store, resource=execution), memory (action=get, resource=execution), memory (action=store, resource=quality_gate), memory (action=get, resource=quality_gate), memory (action=store, resource=error_pattern), memory (action=get, resource=error_pattern) | 10 |
| Agent Sessions | session (action=create), session (action=get), session (action=update), session (action=list), agent (action=log_tool), agent (action=log_delegation) | 6 |
| Project Workflow | project_* (9 tools) | 9 |
| **Total** | | **38** |

## Decision

Consolidate to **8 tools** using resource-action parameterization pattern:

### New Tool Architecture

| Tool | Replaces | Pattern |
| ------ | ---------- | --------- |
| `index` | index (action=start), index (action=status), index (action=clear) | action: start, status, clear |
| `search` | search (resource=code), search (resource=memory), memory (action=list, resource=observation) | resource: code, memory; mode: semantic, keyword |
| `validate` | validate (action=run, scope=project), validate (action=run, scope=file), validate (action=list_rules), validate (action=list_rules), validate (action=analyze) | action: run, list_rules; scope: file, project |
| `memory` | memory (action=store, resource=observation), memory (action=timeline, resource=observation), memory (action=get, resource=observation), memory (action=inject, resource=observation), memory (action=store, resource=execution), memory (action=get, resource=execution), memory (action=store, resource=quality_gate), memory (action=get, resource=quality_gate), memory (action=store, resource=error_pattern), memory (action=get, resource=error_pattern) | action: store, get, timeline, inject; resource: observation, execution, quality_gate, error_pattern |
| `session` | session (action=summarize), session (action=summarize), session (action=create), session (action=get), session (action=update), session (action=list) | action: create, get, update, list, summarize |
| `agent` | agent (action=log_tool), agent (action=log_delegation) | action: log_tool, log_delegation |
| `project` | project_* (9 tools) | action: create, update, list; resource: phase, issue, dependency, decision |
| `vcs` | vcs_* (5 tools) | action: list_repositories, index_repository, compare_branches, search_branch, analyze_impact |

### Tool Schemas

```rust
// 1. index - Codebase indexing operations
struct IndexArgs {
    action: IndexAction,  // start, status, clear
    path: Option<String>,
    collection: Option<String>,
}

// 2. search - Unified search across code and memory
struct SearchArgs {
    query: String,
    resource: SearchResource,  // code, memory
    filters: Option<SearchFilters>,
    limit: Option<u32>,
}

// 3. validate - Architecture and code validation
struct ValidateArgs {
    action: ValidateAction,  // run, list_rules, analyze
    scope: Option<ValidateScope>,  // file, project
    path: Option<String>,
    rules: Option<Vec<String>>,
}

// 4. memory - Unified memory operations
struct MemoryArgs {
    action: MemoryAction,  // store, get, list, timeline, inject
    resource: MemoryResource,  // observation, execution, quality_gate, error_pattern
    data: Option<serde_json::Value>,
    filters: Option<MemoryFilters>,
}

// 5. session - Session management
struct SessionArgs {
    action: SessionAction,  // create, get, update, list, summarize
    session_id: Option<String>,
    data: Option<serde_json::Value>,
    filters: Option<SessionFilters>,
}

// 6. agent - Agent activity logging
struct AgentArgs {
    action: AgentAction,  // log_tool, log_delegation
    session_id: String,
    data: serde_json::Value,
}

// 7. project - Project workflow management
struct ProjectArgs {
    action: ProjectAction,  // create, update, list, delete
    resource: ProjectResource,  // phase, issue, dependency, decision
    project_id: String,
    data: Option<serde_json::Value>,
    filters: Option<ProjectFilters>,
}
```

### Migration Path

1. **Phase 1**: Add new tools alongside existing
2. **Phase 2**: Deprecate old tools (mark as [DEPRECATED] in description)
3. **Phase 3**: Remove deprecated tools after 1 release cycle

### Tool Count Reduction

| Category | Before | After | Reduction |
| ---------- | -------- | ------- | | ----------- |
| Index | 4 | 1 | -3 |
| Search | 3 | 1 | -2 |
| Validation | 5 | 1 | -4 |
| Memory | 14 | 1 | -13 |
| Sessions | 6 | 2 | -4 |
| Project | 9 | 1 | -8 |
| VCS | 5 | 1 | -4 |
| **Total** | **38** | **8** | **-30 (79% reduction)** |

## Consequences

### Positive

- Dramatically reduced cognitive load (38 → 8 tools)
- Consistent action/resource pattern across all tools
- Easier to add new resources without new tools
- Better discoverability through unified interfaces

### Negative

- Migration effort required
- Slightly more complex individual tool schemas
- Potential breaking change for existing integrations

### Neutral

- Same underlying functionality
- Handler code restructured but logic preserved

## Implementation

See mcb-4vg for implementation tracking.
