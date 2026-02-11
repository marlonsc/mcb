# MCP Consolidation - Quick Reference Card

## 📊 The Numbers

-   **50+** MCP endpoints (8 handlers)
-   **15+** Admin HTTP endpoints
-   **9** single-use response types (dead weight)
-   **10** high-ROI reuse opportunities
-   **~30%** boilerplate reduction possible

## 🎯 Top 3 Immediate Wins

### #1: Index Status Wrapper (30 min) - ⭐⭐⭐ ROI

```
MCP:  IndexHandler::handle(IndexAction::Status) → JSON
Admin: GET /jobs (unified jobs status endpoint)
→ Wrap MCP, eliminate duplication
```

### #2: Collection Search (2 hours) - ⭐⭐⭐ ROI

```
MCP:  SearchHandler::handle(SearchResource::Code) → results
Admin: (Currently no search endpoint)
→ Add GET /collections/:name/search?q=...
→ Enables semantic search UI
```

### #3: Response Type Consolidation (4 hours) - ⭐⭐ ROI

```
Current: AdminHealthResponse, JobsStatusResponse, ... (9 types)
Target:  ApiResponse<T> wrapper
→ Reduces boilerplate by 30%
→ Consistent error handling
```

## 📋 Handler Patterns

### MCP (Standard Pattern)

```rust
pub struct IndexHandler { service: Arc<dyn ServiceInterface> }
pub async fn handle(&self, Parameters(args): Parameters<Args>) -> Result<CallToolResult, McpError>
```

### Admin HTTP (Rocket)

```rust
#[get("/endpoint")]
pub fn handler(_auth: AdminAuth, state: &State<AdminState>) -> Json<Response>
```

### Key Differences

| Aspect | MCP | Admin |
|--------|-----|-------|
| Transport | MCP protocol | HTTP |
| Return | `CallToolResult` | `Json<T>` |
| Errors | `McpError` | `(Status, Json<Error>)` |
| Routing | Action enum | Rocket decorators |
| Auth | Protocol-level | `AdminAuth` guard |

## 🔗 Alignment Matrix (MCP ↔ Admin)

| Handler | Endpoint Ready | Response Match | Notes |
|---------|---|---|---|
| **INDEX** | ✅✅ | ✅ | Can wrap directly |
| **SEARCH** | ✅✅ | ✅ | New feature |
| **VALIDATE** | ✅ | ✅ | Needs HTTP wrapper |
| **MEMORY** | ⚠️ | ✅ | New debug endpoints |
| **SESSION** | ⚠️ | ⚠️ | Domain model mismatch |
| **VCS** | ⚠️ | ⚠️ | New feature |
| **AGENT** | ⚠️ | ❌ | No admin UI yet |
| **PROJECT** | ❌ | N/A | Not implemented |

Legend: ✅✅ Ready to use | ✅ Minor work | ⚠️ Medium adaptation | ❌ Separate impl

## 📂 Files to Modify (By Phase)

### Phase 1: Quick Wins (4 hrs)

-   [ ] `admin/handlers.rs` - index status wrapper
-   [ ] `admin/search_handlers.rs` - NEW
-   [ ] `admin/validate_handlers.rs` - NEW
-   [ ] `admin/routes.rs` - add routes

### Phase 2: Consolidation (4 hrs)

-   [ ] `admin/models.rs` - `ApiResponse<T>` wrapper
-   [ ] `admin/handlers.rs` - refactor (10+ endpoints)
-   [ ] `admin/lifecycle_handlers.rs` - refactor
-   [ ] `admin/browse_handlers.rs` - refactor

### Phase 3: Extensions (6 hrs)

-   [ ] `admin/memory_handlers.rs` - NEW (memory browsing)
-   [ ] `admin/session_handlers.rs` - NEW (session browsing)
-   [ ] `admin/vcs_handlers.rs` - NEW (VCS browsing)
-   [ ] `admin/routes.rs` - mount new routes

### Phase 4: Strategic (8+ hrs)

-   [ ] `handlers/project.rs` - implement
-   [ ] Create unified service facade
-   [ ] Add pagination + filtering

## 🎭 Response Type Consolidation

### Dead Weight (Single-Use)

```
❌ AdminHealthResponse (only in /health)
❌ Obsolete IndexingStatusResponse (replaced by JobsStatusResponse on /jobs)
❌ ReadinessResponse (only in /ready)
❌ LivenessResponse (only in /live)
❌ ShutdownResponse (only in /shutdown)
❌ CacheErrorResponse (only in /cache/stats error)
❌ ServiceListResponse (only in /services)
❌ ServiceActionResponse (only in /services/:name/*)
❌ ServiceErrorResponse (only in /services errors)
```

### Proposed Wrapper

```rust
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub status: String,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: u64,
}
```

## 🔄 Error Handling Patterns

### MCP Style

```rust
args.validate()
    .map_err(|e| McpError::invalid_params(format!("Invalid: {}", e), None))?;

if query.is_empty() {
    return Ok(CallToolResult::error(vec![Content::text("Empty query")]));
}
```

### Admin HTTP Style

```rust
let Some(resource) = &state.optional else {
    return Err((Status::ServiceUnavailable, Json(error)));
};

match service.operation().await {
    Ok(result) => Ok(Json(result)),
    Err(e) => Err((Status::InternalServerError, Json(error))),
}
```

## 📈 ROI Ranking

### Tier 1: Quick Wins (< 2 hrs, high impact)

1. ⭐⭐⭐ Index Status Wrapper
2. ⭐⭐⭐ Collection Search
3. ⭐⭐ Validation Endpoints
4. ⭐⭐ Complexity Analysis

### Tier 2: Medium Effort (2-4 hrs)

1. ⭐⭐⭐ Response Type Consolidation
2. ⭐⭐ Memory Browsing
3. ⭐⭐ VCS Operations
4. ⭐⭐ Session Browsing

### Tier 3: Strategic (4+ hrs)

1. ⭐⭐⭐ Project Handler
2. ⭐⭐⭐⭐ Unified Facade

## ✅ Success Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Admin reusing MCP | 60% | 0% |
| Single-use types | 0 | 9 |
| Code duplication | <5% | 15% |
| Handler coverage | >90% | TBD |
| API latency | <50ms | TBD |

## 🚀 Next Steps

1. **Review** - Approve consolidation strategy
2. **Break Down** - Create GitHub issues for each phase
3. **Implement** - Start Phase 1 (quick wins)
4. **Test** - Validate each phase
5. **Document** - Update API docs

---

**Full Details**: See `docs/CONSOLIDATION_ANALYSIS.md` and `docs/MCP_CONSOLIDATION_DETAILED.md`
