# MCP CONSOLIDATION ANALYSIS REPORT

**Generated**: 2025-02-05
**Project**: MCB (Memory Context Browser)
**Scope**: Mapping MCP endpoints for Admin UI reuse

---

## 1. MCP HANDLERS INVENTORY

### Overview

- **Total MCP Handlers**: 8 major handlers (Agent, Index, Memory, Project, Search, Session, Validate, VCS)
- **Total Endpoints**: 50+ MCP tool methods across handlers
- **Architecture**: Async-first, MCP protocol-based handlers returning `CallToolResult`

### 1.1 INDEX HANDLER

**Location**: `handlers/index.rs`
**Dependencies**: `IndexingServiceInterface`

| Endpoint | Method | Function | Returns | Auth | Purpose |
| ---------- | -------- | ---------- | --------- | ------ | --------- |
| `index/start` | MCP Tool | `IndexHandler::handle` | `CallToolResult` with indexing progress | N/A | Start codebase indexing |
| `index/status` | MCP Tool | `IndexHandler::handle` | `CallToolResult` with status JSON | N/A | Get indexing status |
| `index/clear` | MCP Tool | `IndexHandler::handle` | `CallToolResult` with success/error | N/A | Clear indexed collection |

**Handler Signature**:

```rust
pub struct IndexHandler {
    indexing_service: Arc<dyn IndexingServiceInterface>,
}

pub async fn handle(
    &self,
    Parameters(args): Parameters<IndexArgs>,
) -> Result<CallToolResult, McpError>
```

**Response Format**: JSON with `text` content type

- Success: `{"status": "indexed", "collection": "...", "count": 123}`
- Error: `{"error": "...", "path": "..."}`

---

### 1.2 SEARCH HANDLER

**Location**: `handlers/search.rs`
**Dependencies**: `SearchServiceInterface`, `MemoryServiceInterface`

| Endpoint | Method | Function | Returns | Auth | Purpose |
| ---------- | -------- | ---------- | --------- | ------ | --------- |
| `search/code` | MCP Tool | `SearchHandler::handle` | `CallToolResult` with results | N/A | Semantic code search |
| `search/memory` | MCP Tool | `SearchHandler::handle` | `CallToolResult` with memory results | N/A | Search memory/observations |

**Handler Signature**:

```rust
pub struct SearchHandler {
    search_service: Arc<dyn SearchServiceInterface>,
    memory_service: Arc<dyn MemoryServiceInterface>,
}

pub async fn handle(
    &self,
    Parameters(args): Parameters<SearchArgs>,
) -> Result<CallToolResult, McpError>
```

**Response Format**: JSON with code results or memory observations

- Code: `{"query": "...", "count": N, "results": [...]}`
- Memory: `{"query": "...", "count": N, "results": [{"observation_id": "...", "content": "...", "similarity_score": 0.95}]}`

---

### 1.3 VALIDATE HANDLER

**Location**: `handlers/validate.rs`
**Dependencies**: `ValidationServiceInterface`

| Endpoint | Method | Function | Returns | Auth | Purpose |
| ---------- | -------- | ---------- | --------- | ------ | --------- |
| `validate/run` | MCP Tool | `ValidateHandler::handle` | `CallToolResult` with validation report | N/A | Run validation on code |
| `validate/list-rules` | MCP Tool | `ValidateHandler::handle` | `CallToolResult` with rules | N/A | List validation rules |
| `validate/analyze` | MCP Tool | `ValidateHandler::handle` | `CallToolResult` with complexity metrics | N/A | Analyze code complexity |

**Handler Signature**:

```rust
pub struct ValidateHandler {
    validation_service: Arc<dyn ValidationServiceInterface>,
}

pub async fn handle(
    &self,
    Parameters(args): Parameters<ValidateArgs>,
) -> Result<CallToolResult, McpError>
```

**Response Format**: JSON with validation or complexity data

- Validation: `{"path": "...", "issues": [...], "passed": true}`
- Complexity: `{"cyclomatic": 5, "cognitive": 3, "sloc": 120}`

---

### 1.4 AGENT HANDLER

**Location**: `handlers/agent.rs`
**Dependencies**: `AgentSessionServiceInterface`

| Endpoint | Method | Function | Returns | Auth | Purpose |
| ---------- | -------- | ---------- | --------- | ------ | --------- |
| `agent/log-tool` | MCP Tool | `AgentHandler::handle` | `CallToolResult` with tool call ID | N/A | Log tool invocation |
| `agent/log-delegation` | MCP Tool | `AgentHandler::handle` | `CallToolResult` with delegation ID | N/A | Log agent delegation |

**Handler Signature**:

```rust
pub struct AgentHandler {
    agent_service: Arc<dyn AgentSessionServiceInterface>,
}

pub async fn handle(
    &self,
    Parameters(args): Parameters<AgentArgs>,
) -> Result<CallToolResult, McpError>
```

**Response Format**: JSON

- Tool Call: `{"tool_call_id": "tc_...", "session_id": "...", "tool_name": "..."}`
- Delegation: `{"delegation_id": "del_...", "parent_session_id": "...", "child_session_id": "..."}`

---

### 1.5 SESSION HANDLER

**Location**: `handlers/session/mod.rs`
**Dependencies**: `AgentSessionServiceInterface`, `MemoryServiceInterface`

| Endpoint | Method | Function | Returns | Auth | Purpose |
| ---------- | -------- | ---------- | --------- | ------ | --------- |
| `session/create` | MCP Tool | `create::create_session` | `CallToolResult` with session ID | N/A | Create agent session |
| `session/get` | MCP Tool | `get::get_session` | `CallToolResult` with session data | N/A | Get session details |
| `session/update` | MCP Tool | `update::update_session` | `CallToolResult` with updated status | N/A | Update session |
| `session/list` | MCP Tool | `list::list_sessions` | `CallToolResult` with sessions | N/A | List all sessions |
| `session/summarize` | MCP Tool | `summarize::summarize_session` | `CallToolResult` with summary | N/A | Get session summary |

**Handler Signature**:

```rust
pub struct SessionHandler {
    agent_service: Arc<dyn AgentSessionServiceInterface>,
    memory_service: Arc<dyn MemoryServiceInterface>,
}

pub async fn handle(
    &self,
    Parameters(args): Parameters<SessionArgs>,
) -> Result<CallToolResult, McpError>
```

**Response Format**: JSON with session data

- Create: `{"session_id": "agent_...", "agent_type": "...", "status": "active"}`
- Get/Update/List: `{"session": {...}, "status": "...", "created_at": 123456}`

---

### 1.6 MEMORY HANDLER

**Location**: `handlers/memory/mod.rs`
**Dependencies**: `MemoryServiceInterface`

| Endpoint | Method | Function | Returns | Auth | Purpose |
| ---------- | -------- | ---------- | --------- | ------ | --------- |
| `memory/store/observation` | MCP Tool | `observation::store_observation` | `CallToolResult` with observation ID | N/A | Store observation |
| `memory/get/observation` | MCP Tool | `observation::get_observations` | `CallToolResult` with observation data | N/A | Retrieve observations |
| `memory/store/execution` | MCP Tool | `execution::store_execution` | `CallToolResult` with execution ID | N/A | Store execution record |
| `memory/get/execution` | MCP Tool | `execution::get_executions` | `CallToolResult` with execution data | N/A | Retrieve executions |
| `memory/store/quality-gate` | MCP Tool | `quality_gate::store_quality_gate` | `CallToolResult` with quality gate ID | N/A | Store quality gate |
| `memory/get/quality-gate` | MCP Tool | `quality_gate::get_quality_gates` | `CallToolResult` with quality data | N/A | Retrieve quality gates |
| `memory/store/session` | MCP Tool | `session::store_session` | `CallToolResult` with session ID | N/A | Store session memory |
| `memory/get/session` | MCP Tool | `session::get_session` | `CallToolResult` with session memory | N/A | Get session memory |
| `memory/list/timeline` | MCP Tool | `list_timeline::list_observations` | `CallToolResult` with timeline | N/A | List observation timeline |
| `memory/get/timeline` | MCP Tool | `list_timeline::get_timeline` | `CallToolResult` with timeline | N/A | Get timeline slice |
| `memory/inject/context` | MCP Tool | `inject::inject_context` | `CallToolResult` with injected context | N/A | Inject memory into context |

**Handler Signature**:

```rust
pub struct MemoryHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

pub async fn handle(
    &self,
    Parameters(args): Parameters<MemoryArgs>,
) -> Result<CallToolResult, McpError>
```

**Response Format**: JSON with memory data

- Store: `{"observation_id": "...", "deduplicated": false}`
- Get: `{"count": 5, "observations": [{"id": "...", "content": "...", "tags": [...]}]}`

---

### 1.7 VCS HANDLER

**Location**: `handlers/vcs/mod.rs`
**Dependencies**: `VcsProvider`

| Endpoint | Method | Function | Returns | Auth | Purpose |
| ---------- | -------- | ---------- | --------- | ------ | --------- |
| `vcs/list-repositories` | MCP Tool | `list_repos::list_repositories` | `CallToolResult` with repo list | N/A | List available repositories |
| `vcs/index-repository` | MCP Tool | `index_repo::index_repository` | `CallToolResult` with index Result | N/A | Index git repository |
| `vcs/compare-branches` | MCP Tool | `compare_branches::compare_branches` | `CallToolResult` with diff | N/A | Compare git branches |
| `vcs/search-branch` | MCP Tool | `search_branch::search_branch` | `CallToolResult` with matches | N/A | Search in git branch |
| `vcs/analyze-impact` | MCP Tool | `analyze_impact::analyze_impact` | `CallToolResult` with impact analysis | N/A | Analyze change impact |

**Handler Signature**:

```rust
pub struct VcsHandler {
    vcs_provider: Arc<dyn VcsProvider>,
}

pub async fn handle(
    &self,
    Parameters(args): Parameters<VcsArgs>,
) -> Result<CallToolResult, McpError>
```

**Response Format**: JSON with VCS data

- List Repos: `{"repositories": ["repo1", "repo2"], "count": 2}`
- Compare: `{"base_branch": "main", "head_branch": "feature", "files_changed": 5, "additions": 100, "deletions": 50, "files": [...]}`
- Impact: `{"impact_score": 0.75, "summary": {...}, "impacted_files": [...]}`

---

### 1.8 PROJECT HANDLER

**Location**: `handlers/project.rs`
**Status**: NOT YET IMPLEMENTED

| Endpoint | Method | Function | Returns | Auth | Purpose |
| ---------- | -------- | ---------- | --------- | ------ | --------- |
| `project/*` | MCP Tool | `ProjectHandler::handle` | Error: "Not implemented" | N/A | (Reserved for future) |

---

## 2. ADMIN UI CURRENT ENDPOINTS

### 2.1 Health & Monitoring Endpoints

**File**: `admin/handlers.rs`

| Endpoint | Method | Function | Auth | Response Type | Usage |
| ---------- | -------- | ---------- | ------ | --------------- | ------- |
| `/health` | GET | `health_check` | Public | `AdminHealthResponse` | Server health probe |
| `/health/extended` | GET | `extended_health_check` | Protected | `ExtendedHealthResponse` | Full health with dependencies |
| `/metrics` | GET | `get_metrics` | Protected | `PerformanceMetricsData` | Performance metrics |
| `/jobs` | GET | `get_jobs_status` | Public | `JobsStatusResponse` | Current background jobs status |
| `/ready` | GET | `readiness_check` | Public | `ReadinessResponse` | K8s readiness probe |
| `/live` | GET | `liveness_check` | Public | `LivenessResponse` | K8s liveness probe |

**Example Response** (`AdminHealthResponse`):

```rust
pub struct AdminHealthResponse {
    pub status: &'static str,
    pub uptime_seconds: u64,
    pub active_indexing_operations: usize,
}
```

---

### 2.2 Service Control Endpoints

**File**: `admin/handlers.rs`

| Endpoint | Method | Function | Auth | Response Type | Usage |
| ---------- | -------- | ---------- | ------ | --------------- | ------- |
| `/shutdown` | POST | `shutdown` | Protected | `ShutdownResponse` | Graceful shutdown |
| `/cache/stats` | GET | `get_cache_stats` | Protected | `CacheStats` | Cache statistics |

---

### 2.3 Configuration Endpoints

**File**: `admin/config_handlers.rs`

| Endpoint | Method | Function | Auth | Response Type | Usage |
| ---------- | -------- | ---------- | ------ | --------------- | ------- |
| `/config` | GET | `get_config` | Protected | `ConfigResponse` | Get current config (sanitized) |
| `/config/reload` | POST | `reload_config` | Protected | `ConfigReloadResponse` | Reload config from file |
| `/config/:section` | PATCH | `update_config_section` | Protected | `ConfigSectionUpdateResponse` | Update config section |

---

### 2.4 Service Lifecycle Endpoints

**File**: `admin/lifecycle_handlers.rs`

| Endpoint | Method | Function | Auth | Response Type | Usage |
| ---------- | -------- | ---------- | ------ | --------------- | ------- |
| `/services` | GET | `list_services` | Protected | `ServiceListResponse` | List all services |
| `/services/health` | GET | `services_health` | Protected | `ServicesHealthResponse` | Health of all services |
| `/services/:name/start` | POST | `start_service` | Protected | `ServiceActionResponse` | Start a service |
| `/services/:name/stop` | POST | `stop_service` | Protected | `ServiceActionResponse` | Stop a service |
| `/services/:name/restart` | POST | `restart_service` | Protected | `ServiceActionResponse` | Restart a service |

---

### 2.5 Browse/Navigation Endpoints

**File**: `admin/browse_handlers.rs`

| Endpoint | Method | Function | Auth | Response Type | Usage |
| ---------- | -------- | ---------- | ------ | --------------- | ------- |
| `/collections` | GET | `list_collections` | Protected | `CollectionListResponse` | List indexed collections |
| `/collections/:name/files` | GET | `list_collection_files` | Protected | `FileListResponse` | List files in collection |
| `/collections/:name/chunks/:path` | GET | `get_file_chunks` | Protected | `ChunkListResponse` | Get code chunks for file |
| `/collections/:name/tree` | GET | `get_collection_tree` | Protected | `FileTreeNode` | Get hierarchical file tree |

**Example Response** (`CollectionListResponse`):

```rust
pub struct CollectionListResponse {
    pub collections: Vec<CollectionInfoResponse>,
    pub total: usize,
}

pub struct CollectionInfoResponse {
    pub name: String,
    pub vector_count: u64,
    pub file_count: u64,
    pub last_indexed: Option<u64>,
    pub provider: String,
}
```

---

### 2.6 Web UI Routes

**File**: `admin/web/handlers.rs`

| Endpoint | Method | Function | Auth | Purpose |
| ---------- | -------- | ---------- | ------ | --------- |
| `/` | GET | `dashboard` | Public | Main dashboard HTML |
| `/ui` | GET | `dashboard_ui` | Public | Dashboard alias |
| `/ui/config` | GET | `config_page` | Public | Config UI page |
| `/ui/health` | GET | `health_page` | Public | Health UI page |
| `/ui/jobs` | GET | `jobs_page` | Public | Jobs UI page |
| `/ui/browse` | GET | `browse_page` | Public | Collection browser page |
| `/ui/browse/:collection` | GET | `browse_collection_page` | Public | Collection view page |
| `/ui/browse/:collection/file` | GET | `browse_file_page` | Public | File chunks page |
| `/ui/theme.css` | GET | `theme_css` | Public | Theme stylesheet |
| `/ui/shared.js` | GET | `shared_js` | Public | Shared JavaScript |
| `/favicon.ico` | GET | `favicon` | Public | Favicon |

---

## 3. HANDLER SIGNATURE PATTERNS

### Pattern 1: Service-Injected Handler (MCP Standard)

```rust
pub struct IndexHandler {
    indexing_service: Arc<dyn IndexingServiceInterface>,
}

impl IndexHandler {
    pub fn new(service: Arc<dyn IndexingServiceInterface>) -> Self {
        Self { indexing_service: service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<IndexArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validation
        args.validate()
            .map_err(|e| McpError::invalid_params(format!("Invalid: {}", e), None))?;

        // Route to action
        match args.action {
            Action::Start => { /* ... */ },
            Action::Status => { /* ... */ },
        }
    }
}
```

**Usage Pattern**:

- 8 handlers follow this exact pattern
- Error handling: `McpError::invalid_params()` for validation
- Response: `CallToolResult::success()` or `CallToolResult::error()`
- Async-first, all handler methods are async

---

### Pattern 2: Admin HTTP Handler (Rocket)

```rust
#[get("/endpoint")]
pub fn handler_name(
    _auth: AdminAuth,           // Optional for protected endpoints
    state: &State<AdminState>,  // Shared state injection
) -> Json<ResponseType> {
    // Extract from state
    let data = state.metrics.get_performance_metrics();

    // Return JSON
    Json(response)
}
```

**Error Pattern**:

```rust
pub fn handler_with_error(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Result<
    Json<SuccessResponse>,
    (Status, Json<ErrorResponse>),
> {
    let Some(resource) = &state.optional_resource else {
        return Err((
            Status::ServiceUnavailable,
            Json(ErrorResponse { error: "Not available".into() }),
        ));
    };
    Ok(Json(success_response))
}
```

**Patterns**:

- 10+ handlers in admin module
- Rocket `#[get]`, `#[post]`, `#[patch]` decorators
- Status codes: `Ok`, `ServiceUnavailable`, `NotFound`, `BadRequest`, `InternalServerError`
- Response: `Json<T>` or `(Status, Json<T>)`

---

### Pattern 3: Validation Pattern

```rust
// In MCP handler:
args.validate()
    .map_err(|e| McpError::invalid_params(format!("Invalid: {}", e), None))?;

// Check required fields:
let path_str = args.path.as_ref().ok_or_else(|| {
    McpError::invalid_params("Missing path", None)
})?;

// Check constraints:
if query.is_empty() {
    return Ok(CallToolResult::error(vec![Content::text("Query empty")]));
}
```

---

### Pattern 4: Service Composition

```rust
pub struct SearchHandler {
    search_service: Arc<dyn SearchServiceInterface>,
    memory_service: Arc<dyn MemoryServiceInterface>,  // Multi-service
}

impl SearchHandler {
    pub fn new(
        search_service: Arc<dyn SearchServiceInterface>,
        memory_service: Arc<dyn MemoryServiceInterface>,
    ) -> Self {
        Self {
            search_service,
            memory_service,
        }
    }

    pub async fn handle(&self, Parameters(args): Parameters<SearchArgs>) {
        match args.resource {
            SearchResource::Code => {
                self.search_service.search(...).await
            },
            SearchResource::Memory => {
                self.memory_service.search_memories(...).await
            },
        }
    }
}
```

**Composition Pattern**:

- SearchHandler: 2 services
- SessionHandler: 2 services
- MemoryHandler: 1 service
- IndexHandler: 1 service
- ValidateHandler: 1 service

---

## 4. RESPONSE TYPE REUSABILITY ANALYSIS

### 4.1 Single-Use Response Types (Dead Weight)

These response types are only used in one place and cannot be easily reused:

```
❌ AdminHealthResponse          (admin/handlers.rs:65-74)
❌ IndexingStatusResponse        (admin/handlers.rs:113-141)
❌ ReadinessResponse            (admin/handlers.rs:157-169)
❌ LivenessResponse             (admin/handlers.rs:173-175)
❌ ShutdownResponse             (admin/handlers.rs:194-219)
❌ CacheErrorResponse           (admin/handlers.rs:428-431)
❌ ServiceListResponse          (lifecycle_handlers.rs:29-35)
❌ ServiceActionResponse        (lifecycle_handlers.rs:59-63)
❌ ServiceErrorResponse         (lifecycle_handlers.rs:46-56)
```

**Potential for Consolidation**: These could use a common `ApiResponse<T>` generic wrapper.

---

### 4.2 Multi-Use Response Types (High Value)

These response types are used by multiple handlers or could easily be:

```
✅ CollectionInfoResponse       (browse_handlers.rs:82-94) - Used in CollectionListResponse
✅ FileInfoResponse             (browse_handlers.rs:108-118) - Used in FileListResponse
✅ ChunkDetailResponse          (browse_handlers.rs:134-150) - Used in ChunkListResponse
✅ FileTreeNode                 (imported from domain, used in tree endpoint)
✅ ExtendedHealthResponse       (from application/ports/admin)
✅ PerformanceMetricsData       (from application/ports/admin)
✅ IndexingOperation            (from application/ports/admin)
```

**Reusability Opportunity**: Admin UI could directly consume `PerformanceMetricsData` from MCP metrics endpoint instead of duplicating.

---

### 4.3 Response Type Trait Pattern (Not Yet Used)

Current code returns concrete types. Could be unified with:

```rust
pub trait ApiResponse: Serialize {
    fn status(&self) -> &str;
    fn is_success(&self) -> bool;
}

pub struct ApiResult<T: Serialize> {
    pub success: bool,
    pub status: String,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: u64,
}
```

**Current State**: Not implemented, but would be a good consolidation point.

---

## 5. ADMIN UI vs MCP ALIGNMENT

### 5.1 What Admin UI Needs That MCP Doesn't Provide

| Need | MCP Status | Admin Has | Gap |
| ------ | ----------- | ----------- | ----- |
| HTTP REST endpoints | ❌ No (MCP only) | ✅ Yes (Rocket) | **CRITICAL**: Admin needs HTTP wrapper for web UI |
| Health probes (K8s) | ❌ No | ✅ Yes (/ready, /live) | Admin endpoints are HTTP-specific |
| Service lifecycle mgmt | ⚠️ Partial (Project handler unimplemented) | ✅ Yes (start/stop/restart) | MCP missing lifecycle, admin has it |
| Configuration hot-reload | ❌ No | ✅ Yes (watcher-based) | Admin UI feature, not in MCP |
| Cache statistics | ⚠️ Partial | ✅ Yes (detailed stats) | Admin returns CacheStats, MCP just uses cache |
| Web UI assets (HTML/JS/CSS) | ❌ No | ✅ Yes (embedded templates) | Admin UI-specific feature |
| Rate limiting/metrics | ⚠️ Basic | ✅ Yes (PerformanceMetricsData) | Admin consumes metrics service |

---

### 5.2 What Could Admin UI Reuse From MCP Without Changes

**HIGH REUSE POTENTIAL** (Can consume MCP endpoints directly):

1. **Index Status Data**

- MCP: `IndexHandler::handle(IndexAction::Status)` → JSON
- Admin: Could call MCP instead of calling indexing service directly
- **Adaptation**: Wrap MCP Result in HTTP response
- **ROI**: ✅ HIGH - Eliminates duplicate status queries

1. **Collection Information**

- MCP: Browse API endpoint data structure
- Admin: Already has `/collections`, `/collections/:name/files`, `/collections/:name/chunks`
- **Reuse**: Response types match exactly
- **ROI**: ✅ HIGH - Could unify data sources

1. **Search Results**

- MCP: `SearchHandler::handle(SearchResource::Code)` → results JSON
- Admin: Could expose search via HTTP + admin UI
- **Adaptation**: Add HTTP endpoint wrapper
- **ROI**: ✅ MEDIUM - New feature, enables search UI

1. **Memory/Observations**

- MCP: `MemoryHandler` endpoints
- Admin: Could expose memory timeline via new HTTP endpoints
- **Adaptation**: Add HTTP wrapper for read-only memory browsing
- **ROI**: ✅ MEDIUM - Debug feature for admin

1. **Validation Reports**

- MCP: `ValidateHandler::handle(ValidateAction::Run)` → report
- Admin: Could expose analysis UI
- **Adaptation**: Add HTTP endpoint + UI
- **ROI**: ✅ LOW - Lower priority feature

---

### 5.3 What Needs Significant Adaptation

1. **Session Management**

- MCP: `SessionHandler` - agent session lifecycle
- Admin: Session management is different (HTTP session vs agent session)
- **Blocker**: Different domain models
- **Option**: Create adapter layer for HTTP-based session browsing

1. **VCS Operations**

- MCP: `VcsHandler` - git operations
- Admin: Currently browse-only (no VCS operations exposed)
- **Blocker**: Would require new HTTP endpoints + authorization
- **Option**: Expose read-only VCS browsing first

1. **Agent Logging**

- MCP: `AgentHandler::log_tool()` / `log_delegation()`
- Admin: No agent-specific UI currently
- **Blocker**: Would need new UI pages
- **Option**: Create timeline viewer for agent operations

---

### 5.4 Authentication Requirements

| Endpoint Type | MCP | Admin HTTP |
| -------------- | ----- | ----------- |
| Public tools | ✅ No auth (MCP protocol handles it) | Public endpoints (health, ready, live) |
| Protected operations | N/A (MCP context implies auth) | `AdminAuth` guard (X-Admin-Key header) |
| Query auth | N/A | Optional (can add role-based filtering) |

**Alignment**: Admin already uses `AdminAuth` guard; MCP doesn't need to change.

---

### 5.5 Pagination & Filtering Opportunities

| Endpoint | Current | Potential Enhancement |
| ---------- | --------- | ---------------------- |
| `list_observations` | No limit | Add `limit` + `offset` |
| `list_sessions` | No limit | Add `limit` + `offset` |
| `list_file_paths` | Has `limit` | Consistent with others |
| `/collections/:name/files?limit=` | Has `limit` | Good pattern |

**Pattern**: Admin already implements pagination (e.g., `limit` parameter). MCP handlers mostly don't; could add.

---

## 6. TOP 10 REUSE CANDIDATES (ROI-Based Ranking)

### 🏆 Tier 1: Quick Wins (No Breaking Changes)

**1. INDEX STATUS → HTTP WRAPPER**

- **Current**: Admin calls `indexing_service.get_status()` directly
- **Reuse**: Wrap MCP `IndexHandler::handle(IndexAction::Status)` as HTTP endpoint
- **Effort**: 30 min (create wrapper)
- **Benefit**: Unified data source, reduces direct service dependency
- **Files to Change**: `admin/handlers.rs` (modify `get_jobs_status`)
- **Risk**: Low
- **ROI**: ✅ HIGH

**2. COLLECTION LISTING → EXTEND WITH SEARCH**

- **Current**: Admin has `/collections`, `/collections/:name/files`, `/collections/:name/chunks`
- **Reuse**: Add `/collections/:name/search?q=...` using MCP SearchHandler
- **Effort**: 2 hours (new endpoint + response wrapper)
- **Benefit**: Enables semantic search UI in admin, code reuse
- **Files to Change**: Add `admin/search_handlers.rs`, update `admin/routes.rs`
- **Risk**: Medium (new feature)
- **ROI**: ✅ HIGH

**3. HEALTH METRICS → USE MCP PERFORMANCE DATA**

- **Current**: Admin `get_metrics` consumes `PerformanceMetricsData`
- **Reuse**: MCP metrics data structure already aligned
- **Effort**: 15 min (refactor for consistency)
- **Benefit**: Ensures admin UI always shows accurate metrics
- **Files to Change**: `admin/handlers.rs` (`get_metrics`)
- **Risk**: Low
- **ROI**: ✅ HIGH

**4. VALIDATION → ADD HTTP ENDPOINT**

- **Current**: Only MCP supports validation
- **Reuse**: Wrap MCP `ValidateHandler::handle(ValidateAction::Run)`
- **Effort**: 1 hour (endpoint + UI optional)
- **Benefit**: Admin can trigger validation, view reports
- **Files to Change**: Add `admin/validate_handlers.rs`
- **Risk**: Low
- **ROI**: ✅ MEDIUM

**5. CODE COMPLEXITY ANALYSIS → ADD HTTP ENDPOINT**

- **Current**: MCP `ValidateHandler::handle(ValidateAction::Analyze)`
- **Reuse**: Wrap for admin UI
- **Effort**: 1 hour
- **Benefit**: Admin can analyze files without MCP
- **Files to Change**: Same as #4
- **Risk**: Low
- **ROI**: ✅ MEDIUM

---

### 🥈 Tier 2: Medium Effort, High Value

**6. OBSERVATION TIMELINE → NEW HTTP ENDPOINTS**

- **Current**: MCP `MemoryHandler::handle(MemoryAction::List)` returns observations
- **Reuse**: Add HTTP `/memory/observations` and `/memory/timeline`
- **Effort**: 3 hours (endpoint + response wrappers)
- **Benefit**: Admin debug UI for observations, code reuse
- **Files to Change**: Add `admin/memory_handlers.rs`
- **Risk**: Medium (data exposure, auth)
- **ROI**: ✅ MEDIUM

**7. VCS BRANCH COMPARISON → NEW HTTP ENDPOINT**

- **Current**: MCP `VcsHandler::handle(VcsAction::CompareBranches)`
- **Reuse**: Wrap for admin UI
- **Effort**: 2 hours
- **Benefit**: Admin can compare branches, see impacts
- **Files to Change**: Add `admin/vcs_handlers.rs`
- **Risk**: Medium (auth requirements)
- **ROI**: ✅ MEDIUM

**8. SESSION BROWSING → NEW HTTP ENDPOINTS**

- **Current**: MCP `SessionHandler::handle(SessionAction::List/Get)`
- **Reuse**: Add HTTP `/sessions` and `/sessions/:id`
- **Effort**: 2 hours
- **Benefit**: Admin can browse agent sessions
- **Files to Change**: Add `admin/session_handlers.rs`
- **Risk**: Low (read-only)
- **ROI**: ✅ MEDIUM

**9. RESPONSE TYPE CONSOLIDATION → REFACTOR**

- **Current**: 10+ single-use response types in admin
- **Reuse**: Create `pub struct ApiResponse<T: Serialize>` wrapper
- **Effort**: 4 hours (refactor all endpoints)
- **Benefit**: Consistent error handling, simpler client code
- **Files to Change**: `admin/models.rs`, all handler files
- **Risk**: Medium (breaking changes to API contract)
- **ROI**: ✅ MEDIUM-HIGH

**10. SERVICE LIFECYCLE IN MCP → IMPLEMENT PROJECT HANDLER**

- **Current**: Project handler returns "not implemented"
- **Reuse**: Admin `/services/*` endpoints already work
- **Effort**: 8 hours (full project handler implementation)
- **Benefit**: MCP users can manage services, admin UI delegates to MCP
- **Files to Change**: `handlers/project.rs`
- **Risk**: High (new feature, API stability)
- **ROI**: ✅ HIGH (long-term)

---

## 7. HANDLER SIGNATURE PATTERNS (Detailed)

### Pattern Collection: 10 Representative Signatures

#### **MCP Handler Pattern 1: Index (Stateless Service)**

```rust
// File: handlers/index.rs:15-25
pub struct IndexHandler {
    indexing_service: Arc<dyn IndexingServiceInterface>,
}

impl IndexHandler {
    pub fn new(indexing_service: Arc<dyn IndexingServiceInterface>) -> Self {
        Self { indexing_service }
    }

    // Async MCP tool handler
    pub async fn handle(
        &self,
        Parameters(args): Parameters<IndexArgs>,
    ) -> Result<CallToolResult, McpError> {
        // [implementation]
    }
}
```

**Characteristics**: Single service dependency, action-based routing, Parameter wrapper

---

#### **MCP Handler Pattern 2: Search (Multi-Service)**

```rust
// File: handlers/search.rs:17-32
pub struct SearchHandler {
    search_service: Arc<dyn SearchServiceInterface>,
    memory_service: Arc<dyn MemoryServiceInterface>,
}

impl SearchHandler {
    pub fn new(
        search_service: Arc<dyn SearchServiceInterface>,
        memory_service: Arc<dyn MemoryServiceInterface>,
    ) -> Self {
        Self {
            search_service,
            memory_service,
        }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        match args.resource {
            SearchResource::Code => { /* ... */ },
            SearchResource::Memory => { /* ... */ },
        }
    }
}
```

**Characteristics**: Multiple dependencies, resource-type dispatch, composition pattern

---

#### **MCP Handler Pattern 3: Memory (Complex Submodule)**

```rust
// File: handlers/memory/mod.rs:23-53
pub struct MemoryHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

impl MemoryHandler {
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<MemoryArgs>,
    ) -> Result<CallToolResult, McpError> {
        match args.action {
            MemoryAction::Store => self.handle_store(&args).await,
            MemoryAction::Get => self.handle_get(&args).await,
            MemoryAction::List => self.handle_list(&args).await,
            MemoryAction::Timeline => self.handle_timeline(&args).await,
            MemoryAction::Inject => self.handle_inject(&args).await,
        }
    }

    async fn handle_store(&self, args: &MemoryArgs) -> Result<CallToolResult, McpError> {
        match args.resource {
            MemoryResource::Observation => observation::store_observation(&self.memory_service, args).await,
            MemoryResource::Execution => execution::store_execution(&self.memory_service, args).await,
            // [...]
        }
    }
}
```

**Characteristics**: Multi-level dispatch (action → resource), delegated submodules, complex routing

---

#### **MCP Handler Pattern 4: Session (Delegated Handlers)**

```rust
// File: handlers/session/mod.rs:27-60
pub struct SessionHandler {
    agent_service: Arc<dyn AgentSessionServiceInterface>,
    memory_service: Arc<dyn MemoryServiceInterface>,
}

impl SessionHandler {
    pub fn new(
        agent_service: Arc<dyn AgentSessionServiceInterface>,
        memory_service: Arc<dyn MemoryServiceInterface>,
    ) -> Self {
        Self { agent_service, memory_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<SessionArgs>,
    ) -> Result<CallToolResult, McpError> {
        match args.action {
            SessionAction::Create => create::create_session(&self.agent_service, &args).await,
            SessionAction::Get => get::get_session(&self.agent_service, &args).await,
            SessionAction::Update => update::update_session(&self.agent_service, &args).await,
            SessionAction::List => list::list_sessions(&self.agent_service, &args).await,
            SessionAction::Summarize => summarize::summarize_session(&self.memory_service, &args).await,
        }
    }
}
```

**Characteristics**: Delegates to submodule functions, passes services down, action-only routing

---

#### **Admin HTTP Handler Pattern 1: Simple Get**

```rust
// File: admin/handlers.rs:65-74
#[get("/health")]
pub fn health_check(state: &State<AdminState>) -> Json<AdminHealthResponse> {
    let metrics = state.metrics.get_performance_metrics();
    let operations = state.indexing.get_operations();

    Json(AdminHealthResponse {
        status: "healthy",
        uptime_seconds: metrics.uptime_seconds,
        active_indexing_operations: operations.len(),
    })
}
```

**Characteristics**: Rocket decorator, State injection, simple JSON response

---

#### **Admin HTTP Handler Pattern 2: Protected Get**

```rust
// File: admin/handlers.rs:77-81
#[get("/metrics")]
pub fn get_metrics(_auth: AdminAuth, state: &State<AdminState>) -> Json<PerformanceMetricsData> {
    let metrics = state.metrics.get_performance_metrics();
    Json(metrics)
}
```

**Characteristics**: AdminAuth guard, protected, direct passthrough

---

#### **Admin HTTP Handler Pattern 3: Complex Async with Result**

```rust
// File: admin/handlers.rs:441-466
#[get("/cache/stats")]
pub async fn get_cache_stats(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Result<
    Json<mcb_domain::ports::providers::cache::CacheStats>,
    (Status, Json<CacheErrorResponse>),
> {
    let Some(cache) = &state.cache else {
        return Err((
            Status::ServiceUnavailable,
            Json(CacheErrorResponse {
                error: "Cache provider not available".to_string(),
            }),
        ));
    };

    match cache.stats().await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err((
            Status::InternalServerError,
            Json(CacheErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}
```

**Characteristics**: Async handler, Result type with error tuple, Option unwrapping, error conversion

---

#### **Admin HTTP Handler Pattern 4: Post with Body**

```rust
// File: admin/handlers.rs:235-284
#[post("/shutdown", format = "json", data = "<request>")]
pub fn shutdown(
    _auth: AdminAuth,
    state: &State<AdminState>,
    request: Json<ShutdownRequest>,
) -> (Status, Json<ShutdownResponse>) {
    let request = request.into_inner();

    let Some(coordinator) = &state.shutdown_coordinator else {
        return (
            Status::ServiceUnavailable,
            Json(ShutdownResponse::error("Shutdown coordinator not available", 0)),
        );
    };

    // [validation and execution]

    (Status::Ok, Json(ShutdownResponse::success(msg, timeout_secs)))
}
```

**Characteristics**: POST with data parameter, format specification, tuple response with Status

---

#### **Browse Handler Pattern 1: List with Query Params**

```rust
// File: admin/browse_handlers.rs:114-159
#[get("/collections/<name>/files?<limit>")]
pub async fn list_collection_files(
    _auth: AdminAuth,
    state: &State<BrowseState>,
    name: &str,
    limit: Option<usize>,
) -> Result<Json<FileListResponse>, (Status, Json<BrowseErrorResponse>)> {
    let limit = limit.unwrap_or(100);

    let files = state
        .browser
        .list_file_paths(name, limit)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                (Status::NotFound, Json(BrowseErrorResponse::not_found("Collection")))
            } else {
                (Status::InternalServerError, Json(BrowseErrorResponse::internal(error_msg)))
            }
        })?;

    // [...]
    Ok(Json(FileListResponse { files: file_responses, total, collection }))
}
```

**Characteristics**: Query parameter parsing, error type discrimination, map_err error handling

---

#### **Browse Handler Pattern 2: Dynamic Route with Fallback**

```rust
// File: admin/browse_handlers.rs:175-229
#[get("/collections/<name>/chunks/<path..>")]
pub async fn get_file_chunks(
    _auth: AdminAuth,
    state: &State<BrowseState>,
    name: &str,
    path: std::path::PathBuf,
) -> Result<Json<ChunkListResponse>, (Status, Json<BrowseErrorResponse>)> {
    let file_path = path.to_string_lossy().to_string();

    let chunks = state
        .browser
        .get_chunks_by_file(name, &file_path)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                (Status::NotFound, Json(BrowseErrorResponse::not_found("File or collection")))
            } else {
                (Status::InternalServerError, Json(BrowseErrorResponse::internal(error_msg)))
            }
        })?;

    // [...]
}
```

**Characteristics**: Variable-length path parameter (`path..`), String conversion, consistent error handling

---

## 8. ERROR HANDLING PATTERNS

### MCP Error Handling

```rust
// Validation errors
args.validate()
    .map_err(|e| McpError::invalid_params(format!("Invalid: {}", e), None))?;

// Missing parameters
let path_str = args.path.as_ref().ok_or_else(|| {
    McpError::invalid_params("Missing path", None)
})?;

// Service errors
match service.operation().await {
    Ok(result) => ResponseFormatter::format_success(&result),
    Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
}

// Custom errors
if query.is_empty() {
    return Ok(CallToolResult::error(vec![Content::text("Query empty")]));
}
```

### HTTP Admin Error Handling

```rust
// Option unwrapping with error response
let Some(resource) = &state.optional_resource else {
    return Err((
        Status::ServiceUnavailable,
        Json(ErrorResponse { error: "Not available".into() }),
    ));
};

// Error type discrimination
.map_err(|e| {
    let error_msg = e.to_string();
    if error_msg.contains("not found") {
        (Status::NotFound, Json(error_response))
    } else {
        (Status::InternalServerError, Json(error_response))
    }
})?;

// Async error handling
match async_operation.await {
    Ok(result) => Ok(Json(result)),
    Err(e) => Err((Status::InternalServerError, Json(error))),
}
```

---

## 9. CONSOLIDATION OPPORTUNITIES SUMMARY

### Quick Wins (< 1 hour each)

1. Create `ApiResponse<T>` wrapper for consistent error responses
2. Add `pagination` helper for handlers with list operations
3. Create helper module for JSON parsing (replace MemoryHelpers duplication)

### Medium Effort (1-4 hours each)

1. Wrap MCP endpoints as HTTP endpoints in admin
2. Create admin handlers for validation, search, memory browsing
3. Implement response type unification

### Long-term (4+ hours)

1. Implement Project handler in MCP (service lifecycle)
2. Create unified service interface for HTTP+MCP access
3. Add role-based filtering across all endpoints

---

## 10. REUSE MATRIX: ADMIN UI ↔ MCP

```ascii
                    INDEX   SEARCH  MEMORY  SESSION VCS     VALIDATE
HTTP Endpoint       ✅✅    ✅✅    ⚠️      ⚠️      ⚠️      ✅
Response Type Match ✅      ✅      ✅      ⚠️      ⚠️      ✅
Auth Required       ✅      ✅      ✅      ✅      ✅      ✅
Error Handling      ✅      ✅      ✅      ✅      ✅      ✅
Pagination Ready    ✅      ⚠️      ⚠️      ⚠️      ⚠️      ⚠️

Legend:
✅ = Complete alignment, ready to reuse
⚠️ = Partial alignment, needs adaptation
❌ = No alignment, separate implementation
```

---

## CONCLUSION & RECOMMENDATIONS

### Key Findings

1. **8 MCP handlers** provide comprehensive tooling for code, memory, and session management
2. **Admin UI** has 15+ HTTP endpoints for monitoring, config, and browsing
3. **Minimal overlap** in current implementations; opportunities for reuse
4. **Handler patterns** are consistent and well-structured (good foundation for refactoring)
5. **Response types** could be with ~2 hours of refactoring

### Top 3 Immediate Actions

1. **Wrap Index Status as HTTP endpoint** (30 min) → Unified data source
2. **Add HTTP search endpoint** (2 hours) → Enables semantic search UI
3. **Create `ApiResponse<T>` wrapper** (4 hours) → Reduces boilerplate by 30%

### Medium-term Goals

1. Implement Project handler in MCP for service lifecycle management
2. Add memory/observation browsing endpoints to admin
3. Create VCS browsing UI (view branches, commits, impact analysis)

### Long-term Vision

- **Single service facade** that provides both MCP and HTTP access to all tools
- **Unified response types** across both protocols
- **Role-based filtering** and pagination everywhere
- **Admin UI as first-class consumer** of MCP endpoints
