# Integrated Context System (v0.4.0)

## Overview

The Integrated Context System in v0.4.0 introduces a knowledge graph-based approach to code understanding, enabling freshness tracking, time-travel queries, and policy-driven context discovery. This system builds on the workflow FSM foundation (Phase 8) to provide intelligent, adaptive code search and analysis.

## Core Concepts

### 1. Freshness Tracking

Code context degrades over time as repositories evolve. The freshness system tracks:

- **Temporal Metadata**: Last modified timestamps, commit history, branch information
- **Staleness Signals**: Deprecated APIs, outdated patterns, version mismatches
- **Freshness Policies**: Rules for acceptable staleness by context type (e.g., "API docs must be < 7 days old")

**Example Workflow**:

```
User Query: "How do I authenticate users?"
  ↓
Search finds 3 matching code patterns
  ↓
Freshness check: Pattern A (2 days old) ✓, Pattern B (45 days old) ⚠, Pattern C (6 months old) ✗
  ↓
Return Pattern A + B with staleness warnings
```

See **ADR-035: Freshness Tracking** for design details.

### 2. Time-Travel Queries

Understand code evolution by querying historical snapshots:

- **Snapshot Versioning**: Capture code state at specific commits/dates
- **Temporal Queries**: "Show me how this function evolved over 6 months"
- **Regression Detection**: Identify when patterns were introduced/removed

**Example**:

```
Query: "Show authentication patterns from v0.2.0"
  ↓
System retrieves snapshot from v0.2.0 tag
  ↓
Returns code patterns as they existed then
  ↓
Compare with current patterns to show evolution
```

See **ADR-045: Context Versioning** for implementation details.

### 3. Compensation & Rollback

When context becomes stale or invalid, the system can:

- **Detect Invalidation**: Policy violations, breaking changes, deprecated APIs
- **Trigger Compensation**: Refresh context, notify users, suggest alternatives
- **Rollback**: Revert to previous valid context snapshot

**Example Workflow**:

```
Context: "Use OAuth2 for auth"
  ↓
Breaking change detected: OAuth2 endpoint deprecated
  ↓
Compensation triggered: Fetch new OAuth2 endpoint, update context
  ↓
User notified: "Context updated - OAuth2 endpoint changed"
```

See **ADR-037: Compensation & Orchestration** for orchestration patterns.

## Architecture

### 5-Layer Context System

```
┌─────────────────────────────────────────────────────┐
│ Layer 5: Integration & Policies                     │
│ (Policy enforcement, compensation triggers)         │
└─────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────┐
│ Layer 4: Versioning & Snapshots                     │
│ (Context snapshots, temporal queries)               │
└─────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────┐
│ Layer 3: Hybrid Search Engine                       │
│ (RRF fusion, semantic + keyword search)             │
└─────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────┐
│ Layer 2: Knowledge Graph                            │
│ (Code relationships, freshness metadata)            │
└─────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────┐
│ Layer 1: Code Indexing & Embeddings                 │
│ (AST parsing, vector embeddings, storage)           │
└─────────────────────────────────────────────────────┘
```

### Key Components

**CodeGraph** (petgraph-based):

- Nodes: Code entities (functions, classes, modules)
- Edges: Relationships (calls, imports, extends, implements)
- Metadata: Freshness, version, staleness signals

**HybridSearchEngine**:

- Semantic search via embeddings
- Keyword search via full-text index
- RRF (Reciprocal Rank Fusion) for Result ranking
- Freshness filtering and sorting

**ContextSnapshot**:

- Immutable capture of code state at specific commit/date
- Includes graph, embeddings, metadata
- Enables time-travel queries and regression detection

## Workflows

### Workflow 1: Freshness-Aware Search

```
1. User submits query: "How to handle errors?"
2. System searches code graph + embeddings
3. Results ranked by:
   -   Semantic relevance (embedding similarity)
   -   Keyword match (TF-IDF)
   -   Freshness score (recency + staleness signals)
4. Return top results with freshness metadata
5. User can filter by freshness threshold
```

### Workflow 2: Time-Travel Query

```
1. User asks: "Show me error handling from v0.2.0"
2. System retrieves snapshot for v0.2.0 tag
3. Searches within that snapshot's graph
4. Returns historical patterns
5. Optionally compare with current patterns
```

### Workflow 3: Policy-Driven Context Discovery

```
1. Policy defined: "API docs must be < 7 days old"
2. User searches for API documentation
3. System applies policy filter during search
4. Results filtered to only include fresh docs
5. If no fresh results, trigger compensation:
   -   Refresh docs from source
   -   Notify user of update
   -   Cache new version
```

### Workflow 4: Compensation & Rollback

```
1. Context in use: "Use endpoint /api/v1/auth"
2. Breaking change detected: Endpoint deprecated
3. System triggers compensation:
   -   Fetch new endpoint: /api/v2/auth
   -   Update context snapshot
   -   Notify dependent systems
4. If compensation fails:
   -   Rollback to previous snapshot
   -   Mark context as invalid
   -   Suggest manual review
```

## Integration Points

### With Workflow FSM (Phase 8)

The Integrated Context System integrates with the Workflow FSM:

- **FSM Gates**: Context freshness gates workflow transitions
- **Policy Enforcement**: Policies applied at FSM state boundaries
- **Compensation Hooks**: FSM triggers compensation on policy violations

See **ADR-034: Workflow FSM** for FSM details.

### With MCP Tools

New MCP tools expose context system capabilities:

- `search_code`: Semantic search with freshness filtering
- `get_context_snapshot`: Retrieve historical context
- `apply_policy`: Apply freshness/validation policies
- `trigger_compensation`: Manually trigger compensation

## Configuration

### Freshness Policies

```toml
[freshness]

# Default staleness threshold (days)
default_max_age = 30

# Per-context-type policies
[freshness.policies]
api_docs = { max_age = 7, signal = "deprecated" }
examples = { max_age = 14, signal = "outdated" }
patterns = { max_age = 60, signal = "legacy" }
```

### Snapshot Retention

```toml
[snapshots]

# Keep snapshots for last N commits
retention_commits = 100

# Keep snapshots for last N days
retention_days = 90

# Snapshot frequency (commits)
frequency = 10
```

## Examples

### Example 1: Search with Freshness

```bash

# Search for authentication patterns, only fresh results
mcb search --query "authenticate user" --freshness-max-age 7

# Returns:

# 1. OAuth2 implementation (2 days old) ✓

# 2. JWT pattern (5 days old) ✓

# 3. Session-based auth (45 days old) ⚠ [STALE]
```

### Example 2: Time-Travel Query

```bash

# Show authentication patterns from v0.2.0
mcb search --query "authenticate" --snapshot v0.2.0

# Compare with current
mcb search --query "authenticate" --snapshot v0.2.0 --compare-current

# Output shows evolution:

# v0.2.0: Session-based auth

# v0.3.0: Added JWT support

# v0.4.0: OAuth2 + JWT + Session (multi-strategy)
```

### Example 3: Policy-Driven Search

```bash

# Apply "API docs must be fresh" policy
mcb search --query "API reference" --policy api_docs

# Only returns docs < 7 days old

# If no fresh docs found, triggers compensation
```

## Related Documentation

- **ADR-034**: Workflow FSM – State machine for context workflows
- **ADR-035**: Freshness Tracking – Temporal metadata and staleness signals
- **ADR-036**: Policies & Validation – Policy enforcement framework
- **ADR-037**: Compensation & Orchestration – Rollback and recovery patterns
- **ADR-041**: Context Architecture – System design and layers
- **ADR-042**: Knowledge Graph – Graph structure and relationships
- **ADR-043**: Hybrid Search – Search engine design
- **ADR-044**: Model Selection – Embedding and search model choices
- **ADR-045**: Context Versioning – Snapshot and temporal query design
- **ADR-046**: Integration Patterns – MCP tool integration

## Next Steps

1. Review ADR-034-037 for workflow and policy foundations
2. Review ADR-041-046 for context system implementation
3. See [`docs/implementation/phase-9-roadmap.md`](../implementation/phase-9-roadmap.md) for 4-week execution plan
4. Check [`docs/migration/v0.3-to-v0.4.md`](../migration/v0.3-to-v0.4.md) for upgrade guide
