# Handlebars Library - Internal Usage Mapping

**Library**: `handlebars` (v4.x)  
**Status**: IMPLEMENTED (v0.2.0)  
**ADR Reference**: [ADR-007: Integrated Web Administration Interface](../../docs/adr/007-integrated-web-administration-interface.md)  
**Purpose**: Server-side template rendering for admin web UI and code browser

## Architecture Overview

Handlebars is integrated as the **primary templating engine** for MCB's web administration interface. It provides server-side rendering of HTML templates with custom helpers for formatting, badges, and display logic.

### Design Pattern
- **Pattern**: Engine trait + Helper registration
- **Scope**: Admin web UI, code browser, dashboard pages
- **Integration**: Rocket HTTP server, custom helpers

---

## Core Template Engine

### 1. Engine Trait Implementation
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/templates/engine/mod.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 4 | `use handlebars::Handlebars;` | Core handlebars import |
| 11-16 | `Engine` trait | Generic template engine interface |
| 18-25 | `Engines` struct | Container for Handlebars instance |
| 24 | `pub handlebars: Handlebars<'static>` | Public handlebars field |
| 30-39 | `Engines::init()` | Initialize handlebars with templates |
| 37 | `<Handlebars<'static> as Engine>::init(...)` | Trait-based initialization |
| 41-52 | `Engines::render()` | Render template with context |
| 48 | `Engine::render(&self.handlebars, name, context)` | Trait-based rendering |
| 54-61 | `Engines::templates()` | Iterator over registered templates |
| 56-59 | `self.handlebars.get_templates()` | Template enumeration |

### 2. Handlebars Engine Implementation
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/templates/engine/handlebars_engine.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 3 | `use handlebars::Handlebars;` | Handlebars import |
| 8 | `impl Engine for Handlebars<'static>` | Trait implementation |
| 9 | `const EXT: &'static str = "hbs";` | File extension constant |
| 11-24 | `init()` | Template registration from files |
| 12 | `let mut hb = Handlebars::new();` | Handlebars instantiation |
| 15 | `hb.register_template_file(name, path)` | Template file registration |
| 26-36 | `render()` | Template rendering with context |
| 32 | `Handlebars::render(self, name, &context)` | Rendering call |

---

## Custom Helpers

### 3. Helper Registration
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/web/helpers.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 1-5 | Module documentation | Custom helpers for admin UI |
| 8-10 | Imports | Handlebars helper traits and types |
| 381 | `pub fn register_helpers(hbs: &mut Handlebars<'static>)` | Helper registration function |
| 397-402 | Test helper registration | Example usage in tests |

### 4. Timestamp Helper
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/web/helpers.rs:13-42`

| Line | Component | Purpose |
|------|-----------|---------|
| 18-19 | `struct TimestampHelper;` | Helper struct |
| 21-42 | `impl HelperDef for TimestampHelper` | Helper implementation |
| 30 | `h.param(0).and_then(\|p\| p.value().as_i64())` | Parameter extraction |
| 35-37 | DateTime formatting | `DateTime::<Utc>::from_timestamp()` |

### 5. Relative Time Helper
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/web/helpers.rs:44-97`

| Line | Component | Purpose |
|------|-----------|---------|
| 56 | `struct RelativeTimeHelper;` | Helper struct |
| 58-97 | `impl HelperDef for RelativeTimeHelper` | Helper implementation |
| 74-75 | Time delta calculation | `Utc::now().timestamp().saturating_sub(val)` |
| 77-92 | Relative time formatting | Minutes, hours, days, full date |

### 6. JSON Pretty-Print Helper
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/web/helpers.rs:99+`

| Component | Purpose |
|-----------|---------|
| `JsonHelper` | Pretty-prints JSON in `<pre><code>` tags |

---

## Template Integration

### 7. Route Registration
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/routes.rs:74`

| Line | Component | Purpose |
|------|-----------|---------|
| 74 | `crate::admin::web::helpers::register_helpers(&mut engines.handlebars);` | Helper registration in routes |

### 8. Web Router Integration
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/web/router.rs:110`

| Line | Component | Purpose |
|------|-----------|---------|
| 110 | `crate::admin::web::helpers::register_helpers(&mut engines.handlebars);` | Helper registration in router |

### 9. Template Directory Resolution
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/web/router.rs:81,85`

| Line | Component | Purpose |
|------|-----------|---------|
| 81 | `tracing::debug!(template_dir = %candidate, "Resolved template directory");` | Debug logging for template path |
| 85 | `tracing::warn!("No template directory found, using default 'templates'");` | Warning for missing templates |

---

## Web Handler Integration

### 10. Dashboard Rendering
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/web/handlers.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 24 | `tracing::info!("dashboard called");` | Dashboard handler logging |
| 31 | `tracing::info!("dashboard_ui called");` | Dashboard UI handler logging |

### 11. Configuration Page
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/web/handlers.rs:38-52`

| Line | Component | Purpose |
|------|-----------|---------|
| 38 | `tracing::info!("config_page called");` | Config page handler logging |
| 52 | `tracing::info!("config_page called");` | Config page rendering |

### 12. Health Page
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/web/handlers.rs:52-66`

| Line | Component | Purpose |
|------|-----------|---------|
| 52 | `tracing::info!("health_page called");` | Health page handler logging |
| 66 | `tracing::info!("health_page called");` | Health page rendering |

### 13. Jobs Page
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/web/handlers.rs:66-80`

| Line | Component | Purpose |
|------|-----------|---------|
| 66 | `tracing::info!("jobs_page called");` | Jobs page handler logging |
| 80 | `tracing::info!("jobs_page called");` | Jobs page rendering |

### 14. Browse Pages
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/web/handlers.rs:80-146`

| Line | Component | Purpose |
|------|-----------|---------|
| 104 | `tracing::info!("browse_page called");` | Browse page handler logging |
| 118 | `tracing::info!("browse_collection_page called");` | Collection browse handler |
| 132 | `tracing::info!("browse_file_page called");` | File browse handler |
| 146 | `tracing::info!("browse_tree_page called");` | Tree browse handler |

---

## Template Files

### 15. Template Directory Structure
**Location**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/web/templates/`

| Template | Purpose |
|----------|---------|
| `dashboard.hbs` | Main dashboard with metrics |
| `config.hbs` | Configuration management page |
| `health.hbs` | Health status page |
| `jobs.hbs` | Job status page |
| `browse.hbs` | Code browser main page |
| `browse_collection.hbs` | Collection view |
| `browse_file.hbs` | File view with syntax highlighting |
| `browse_tree.hbs` | Tree navigation view |

---

## Cargo.toml Dependencies

### 16. Dependency Declaration
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/Cargo.toml:62`

```toml
handlebars = { workspace = true }
```

**File**: `/home/marlonsc/mcb/crates/mcb-server/Cargo.toml:74`

```toml
handlebars = { workspace = true }
```

**Workspace Definition**: `/home/marlonsc/mcb/Cargo.toml`
- Version: 4.x
- Features: Default

---

## ADR Alignment

### ADR-007: Integrated Web Administration Interface
- **Status**: IMPLEMENTED (v0.2.0)
- **Rationale**:
  - Handlebars chosen for server-side rendering
  - Mature, battle-tested templating engine
  - Excellent Rust integration with type-safe context
  - Custom helpers for domain-specific formatting
- **Key Features**:
  - System Dashboard with real-time metrics
  - Configuration Management interface
  - Provider Management UI
  - Index Management controls
  - Security with JWT authentication
  - Monitoring and alerting
- **Trade-offs**:
  - Server-side rendering (no client-side reactivity)
  - Requires template files in deployment
  - Limited to Rocket HTTP server

### ADR-026: Routing Refactor (Rocket/Poem)
- **Alignment**: Handlebars integrated with Rocket HTTP server
- **Pattern**: Rocket responders for template rendering

---

## Error Handling

### Template Registration Errors
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/templates/engine/handlebars_engine.rs:15-20`

| Line | Component | Purpose |
|------|-----------|---------|
| 15 | `if let Err(e) = hb.register_template_file(name, path)` | Error handling |
| 16 | `error!("Handlebars template '{}' failed to register.", name);` | Error logging |
| 17 | `error_!("{}", e);` | Error details |
| 18 | `info_!("Template path: '{}'.", path.to_string_lossy());` | Path logging |

### Template Rendering Errors
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/templates/engine/handlebars_engine.rs:26-35`

| Line | Component | Purpose |
|------|-----------|---------|
| 27 | `if self.get_template(name).is_none()` | Template existence check |
| 28 | `error_!("Handlebars template '{}' does not exist.", name);` | Missing template error |
| 32 | `.map_err(\|e\| error_!("Handlebars: {}", e))` | Rendering error handling |

---

## Testing & Validation

### 17. Helper Tests
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/web/helpers.rs:397-402`

| Line | Component | Purpose |
|------|-----------|---------|
| 397 | `let mut hbs = Handlebars::new();` | Test handlebars instance |
| 401 | `register_helpers(&mut hbs);` | Helper registration test |
| 402 | `register_helpers(&mut hbs);` | Idempotency test |

---

## Summary Table

| Aspect | Details |
|--------|---------|
| **Core Engine** | `/home/marlonsc/mcb/crates/mcb-server/src/templates/engine/mod.rs:1-62` |
| **Handlebars Impl** | `/home/marlonsc/mcb/crates/mcb-server/src/templates/engine/handlebars_engine.rs:1-37` |
| **Helper Registration** | `/home/marlonsc/mcb/crates/mcb-server/src/admin/web/helpers.rs:381+` |
| **Custom Helpers** | 6+ helpers (timestamp, relative_time, json, etc.) |
| **Route Integration** | `/home/marlonsc/mcb/crates/mcb-server/src/admin/routes.rs:74` |
| **Web Router** | `/home/marlonsc/mcb/crates/mcb-server/src/admin/web/router.rs:110` |
| **Handlers** | 8+ web handlers in `/admin/web/handlers.rs` |
| **Templates** | 8+ template files in `/admin/web/templates/` |
| **ADR** | ADR-007 (primary), ADR-026 |
| **Status** | IMPLEMENTED, production-ready |

