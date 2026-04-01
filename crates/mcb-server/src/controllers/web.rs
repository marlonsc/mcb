//! Web controllers for MCB admin HTML pages served at `/ui/*`.

use crate::controllers::admin_config::load_admin_config;
use crate::state::McbState;
use axum::extract::Extension;
use loco_rs::prelude::*;
use mcb_domain::ports::{IndexingOperationStatus, ValidationStatus};
use mcb_domain::value_objects::CollectionId;

/// Common HTML page layout wrapper.
fn page_layout(title: &str, active: &str, body: &str) -> String {
    let nav_link = |href: &str, label: &str| -> String {
        let class = if label == active {
            " class=\"active\""
        } else {
            ""
        };
        format!(r#"<a href="{href}"{class}>{label}</a>"#)
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="en" data-theme="light">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MCB Admin - {title}</title>
    <link rel="icon" href="/favicon.ico" type="image/svg+xml">
    <link rel="stylesheet" href="/ui/theme.css">
    <script>
        (function() {{
            var saved = localStorage.getItem('mcb-theme');
            if (saved) document.documentElement.setAttribute('data-theme', saved);
        }})();
    </script>
</head>
<body>
    <nav class="main-nav">
        <div class="nav-brand">MCB Admin</div>
        <div class="nav-links">
            {d}{c}{h}{j}{b}
        </div>
        <button title="Toggle Theme" aria-label="Toggle theme" onclick="toggleTheme()">🌓</button>
    </nav>
    <main class="content">
        {body}
    </main>
    <script src="/ui/shared.js"></script>
    <script>
        function toggleTheme() {{
            var html = document.documentElement;
            var current = html.getAttribute('data-theme') || 'light';
            var next = current === 'light' ? 'dark' : (current === 'dark' ? 'auto' : 'light');
            html.setAttribute('data-theme', next);
            localStorage.setItem('mcb-theme', next);
        }}
    </script>
</body>
</html>"#,
        d = nav_link("/ui/", "Dashboard"),
        c = nav_link("/ui/config", "Configuration"),
        h = nav_link("/ui/health", "Health"),
        j = nav_link("/ui/jobs", "Jobs"),
        b = nav_link("/ui/browse", "Browse"),
    )
}

/// Dashboard page — main admin landing page with real metrics.
///
/// # Errors
///
/// Fails when dashboard data cannot be loaded.
pub async fn dashboard(Extension(state): Extension<McbState>) -> Result<Response> {
    // Fetch real data from dashboard port
    let stats = state.dashboard.get_agent_session_stats().await.ok();
    let tool_calls = state
        .dashboard
        .get_tool_call_counts()
        .await
        .unwrap_or_default();
    let daily = state
        .dashboard
        .get_observations_by_day(7)
        .await
        .unwrap_or_default();

    let total_sessions = stats.as_ref().map_or(0, |s| s.total_sessions);
    let total_agents = stats.as_ref().map_or(0, |s| s.total_agents);

    // Indexing status
    let indexing_ops = state.indexing_ops.get_operations();
    let validation_ops = state.validation_ops.get_operations();
    let idx_running = indexing_ops
        .values()
        .filter(|op| {
            matches!(
                op.status,
                IndexingOperationStatus::Starting | IndexingOperationStatus::InProgress
            )
        })
        .count();
    let val_running = validation_ops
        .values()
        .filter(|op| {
            matches!(
                op.status,
                ValidationStatus::Queued | ValidationStatus::InProgress
            )
        })
        .count();

    // Provider health
    let emb_ok = state.embedding_provider.health_check().await.is_ok();
    let vec_ok = state.vector_store.health_check().await.is_ok();
    let health_class = if emb_ok && vec_ok {
        "healthy"
    } else {
        "degraded"
    };

    // Top tool calls table
    let tool_rows: String = tool_calls
        .iter()
        .take(8)
        .map(|t| {
            format!(
                "<tr><td>{}</td><td class=\"num\">{}</td></tr>",
                html_escape(&t.tool_name),
                t.count
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Recent observations sparkline (7 days)
    let obs_rows: String = daily
        .iter()
        .map(|d| {
            format!(
                "<tr><td>{}</td><td class=\"num\">{}</td></tr>",
                html_escape(&d.day),
                d.count
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let body = format!(
        r#"
        <h1>Dashboard</h1>
        <div class="dashboard-grid">
            <div class="card metric-card">
                <h3>Sessions</h3>
                <div class="metric-value">{total_sessions}</div>
                <div class="metric-label">total sessions</div>
            </div>
            <div class="card metric-card">
                <h3>Agents</h3>
                <div class="metric-value">{total_agents}</div>
                <div class="metric-label">unique agents</div>
            </div>
            <div class="card metric-card">
                <h3>System Health</h3>
                <div class="metric-value"><span class="status-badge {health_class}">{health_class}</span></div>
                <div class="metric-label">embedding + vector store</div>
            </div>
            <div class="card metric-card">
                <h3>Active Jobs</h3>
                <div class="metric-value">{active_jobs}</div>
                <div class="metric-label">{idx_running} indexing, {val_running} validation</div>
            </div>
        </div>
        <div class="dashboard-grid two-col">
            <div class="card">
                <h3>Tool Usage</h3>
                {tool_table}
            </div>
            <div class="card">
                <h3>Observations (Last 7 Days)</h3>
                {obs_table}
            </div>
        </div>
    "#,
        active_jobs = idx_running + val_running,
        tool_table = if tool_rows.is_empty() {
            "<p class=\"muted\">No tool calls recorded yet.</p>".to_owned()
        } else {
            format!(
                "<table class=\"data-table\"><thead><tr><th>Tool</th><th>Calls</th></tr></thead><tbody>{tool_rows}</tbody></table>"
            )
        },
        obs_table = if obs_rows.is_empty() {
            "<p class=\"muted\">No observations recorded yet.</p>".to_owned()
        } else {
            format!(
                "<table class=\"data-table\"><thead><tr><th>Day</th><th>Count</th></tr></thead><tbody>{obs_rows}</tbody></table>"
            )
        },
    );
    format::html(&page_layout("Dashboard", "Dashboard", &body))
}

/// Configuration page — shows current MCB configuration.
///
/// # Errors
///
/// Fails when config cannot be loaded.
pub async fn config_page(Extension(state): Extension<McbState>) -> Result<Response> {
    let provider_name = state.embedding_provider.provider_name();
    let dimensions = state.embedding_provider.dimensions();
    let admin_config = load_admin_config()?;

    let vector_host = admin_config
        .get("providers")
        .and_then(|v| v.get("vector_store"))
        .and_then(|v| v.get("host"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("localhost");
    let vector_port = admin_config
        .get("providers")
        .and_then(|v| v.get("vector_store"))
        .and_then(|v| v.get("port"))
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(i64::from(
            mcb_utils::constants::vector_store::MILVUS_DEFAULT_PORT,
        ));
    let vector_provider = admin_config
        .get("providers")
        .and_then(|v| v.get("vector_store"))
        .and_then(|v| v.get("provider"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let db_url = admin_config
        .get("database")
        .and_then(|v| v.get("uri"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("(default)");

    let body = format!(
        r#"
        <h1>Configuration</h1>
        <div class="config-grid">
            <div class="card">
                <h3>Embedding Provider</h3>
                <table class="data-table">
                    <tr><td>Provider</td><td><strong>{provider_name}</strong></td></tr>
                    <tr><td>Dimensions</td><td>{dimensions}</td></tr>
                </table>
            </div>
            <div class="card">
                <h3>Vector Store</h3>
                <table class="data-table">
                    <tr><td>Provider</td><td><strong>{vector_provider}</strong></td></tr>
                    <tr><td>Host</td><td>{vector_host}</td></tr>
                    <tr><td>Port</td><td>{vector_port}</td></tr>
                </table>
            </div>
            <div class="card">
                <h3>Database</h3>
                <table class="data-table">
                    <tr><td>URI</td><td>{db_url}</td></tr>
                </table>
            </div>
        </div>
    "#
    );
    format::html(&page_layout("Configuration", "Configuration", &body))
}

/// Health page — shows provider health status.
///
/// # Errors
///
/// Fails when health checks fail.
pub async fn health_page(Extension(state): Extension<McbState>) -> Result<Response> {
    let embedding_ok = state.embedding_provider.health_check().await.is_ok();
    let vector_ok = state.vector_store.health_check().await.is_ok();

    let overall = if embedding_ok && vector_ok {
        "healthy"
    } else {
        "degraded"
    };

    fn status_badge(ok: bool) -> &'static str {
        if ok {
            r#"<span class="status-badge healthy">ok</span>"#
        } else {
            r#"<span class="status-badge degraded">error</span>"#
        }
    }

    let body = format!(
        r#"
        <h1>Health</h1>
        <div class="health-status">
            <h2>Status: <span class="status-badge {overall}">{overall}</span></h2>
            <div class="health-cards">
                <div class="card">
                    <h3>Embedding Provider</h3>
                    <p>{emb_badge}</p>
                    <p class="muted">Provider: {provider}</p>
                    <p class="muted">Dimensions: {dims}</p>
                </div>
                <div class="card">
                    <h3>Vector Store</h3>
                    <p>{vec_badge}</p>
                </div>
            </div>
        </div>
    "#,
        emb_badge = status_badge(embedding_ok),
        vec_badge = status_badge(vector_ok),
        provider = state.embedding_provider.provider_name(),
        dims = state.embedding_provider.dimensions(),
    );
    format::html(&page_layout("Health", "Health", &body))
}

/// Jobs page — shows indexing and validation operations with detail.
///
/// # Errors
///
/// Fails when job data cannot be loaded.
pub async fn jobs_page(Extension(state): Extension<McbState>) -> Result<Response> {
    let indexing_ops = state.indexing_ops.get_operations();
    let validation_ops = state.validation_ops.get_operations();

    let running = indexing_ops
        .values()
        .filter(|op| {
            matches!(
                op.status,
                IndexingOperationStatus::Starting | IndexingOperationStatus::InProgress
            )
        })
        .count()
        + validation_ops
            .values()
            .filter(|op| {
                matches!(
                    op.status,
                    ValidationStatus::Queued | ValidationStatus::InProgress
                )
            })
            .count();

    let total = indexing_ops.len() + validation_ops.len();
    let status = if running > 0 { "running" } else { "idle" };

    // Build indexing job rows
    let idx_rows: String = indexing_ops
        .iter()
        .map(|(id, op)| {
            let status_class = match op.status {
                IndexingOperationStatus::Starting | IndexingOperationStatus::InProgress => {
                    "running"
                }
                IndexingOperationStatus::Completed => "completed",
                _ => "error",
            };
            format!(
                r#"<tr>
                    <td>{}</td>
                    <td>indexing</td>
                    <td><span class="status-badge {status_class}">{:?}</span></td>
                    <td class="num">{}</td>
                </tr>"#,
                html_escape(&id.to_string()),
                op.status,
                op.total_files,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Build validation job rows
    let val_rows: String = validation_ops
        .iter()
        .map(|(id, op)| {
            let status_class = match op.status {
                ValidationStatus::Queued | ValidationStatus::InProgress => "running",
                ValidationStatus::Completed => "completed",
                ValidationStatus::Failed(_) => "error",
            };
            format!(
                r#"<tr>
                    <td>{}</td>
                    <td>validation</td>
                    <td><span class="status-badge {status_class}">{:?}</span></td>
                    <td class="num">—</td>
                </tr>"#,
                html_escape(&id.to_string()),
                op.status,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let all_rows = format!("{idx_rows}{val_rows}");
    let job_table = if all_rows.is_empty() {
        "<p class=\"muted\">No operations recorded.</p>".to_owned()
    } else {
        format!(
            r#"<table class="data-table">
                <thead><tr><th>ID</th><th>Type</th><th>Status</th><th>Files</th></tr></thead>
                <tbody>{all_rows}</tbody>
            </table>"#
        )
    };

    let body = format!(
        r#"
        <h1>Jobs</h1>
        <div class="jobs-summary">
            <div class="dashboard-grid">
                <div class="card metric-card">
                    <h3>Status</h3>
                    <div class="metric-value"><span class="status-badge {status}">{status}</span></div>
                </div>
                <div class="card metric-card">
                    <h3>Total</h3>
                    <div class="metric-value">{total}</div>
                </div>
                <div class="card metric-card">
                    <h3>Running</h3>
                    <div class="metric-value">{running}</div>
                </div>
                <div class="card metric-card">
                    <h3>Complete</h3>
                    <div class="metric-value">{complete}</div>
                </div>
            </div>
        </div>
        <div class="card">
            <h3>Operations</h3>
            {job_table}
        </div>
    "#,
        complete = total - running,
    );
    format::html(&page_layout("Jobs", "Jobs", &body))
}

/// Escapes HTML special characters in a string.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Browse page — shows vector store collections with keyboard navigation.
///
/// # Errors
///
/// Fails when collection data cannot be loaded.
pub async fn browse_page(Extension(state): Extension<McbState>) -> Result<Response> {
    let collections = state
        .vector_store
        .list_collections()
        .await
        .map_err(|e| loco_rs::Error::string(&e.to_string()))?;

    let mut all_chunks: Vec<mcb_domain::value_objects::SearchResult> = Vec::new();
    for collection in &collections {
        let id = CollectionId::from_string(&collection.name);
        let vecs = state
            .vector_store
            .list_vectors(&id, mcb_utils::constants::DEFAULT_BROWSE_LIMIT)
            .await
            .map_err(|e| loco_rs::Error::string(&e.to_string()))?;
        all_chunks.extend(vecs);
    }

    let chunks_html: String = if all_chunks.is_empty() {
        r#"<p class="no-chunks">No collections indexed yet. Use the MCP <code>index_repo</code> tool to index a codebase.</p>"#.to_owned()
    } else {
        all_chunks
            .iter()
            .enumerate()
            .map(|(i, chunk)| {
                let lang = html_escape(&chunk.language.to_lowercase());
                let content = html_escape(&chunk.content);
                let file = html_escape(&chunk.file_path);
                let id_attr = html_escape(&chunk.id);
                format!(
                    r#"<div class="code-chunk" data-chunk-id="{}" data-index="{}" data-language="{}" tabindex="0">
  <div class="chunk-header">
    <span class="chunk-file">{}</span>
    <span class="chunk-lang">{}</span>
    <span class="chunk-lines">:{}</span>
  </div>
  <pre class="chunk-content"><code>{}</code></pre>
</div>"#,
                    id_attr, i, lang, file, lang, chunk.start_line, content
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let body = format!(
        r#"
        <h1>Browse</h1>
        <div id="collections-grid" class="collections-grid">
            {chunks_html}
        </div>
        <script>
(function() {{
  function getChunks() {{ return Array.from(document.querySelectorAll('[data-chunk-id]')); }}
  function getActiveIndex(chunks) {{
    var active = document.querySelector('[data-active="true"]');
    if (active) return parseInt(active.dataset.index || '0', 10);
    var focused = document.activeElement;
    if (focused && focused.hasAttribute('data-chunk-id')) return parseInt(focused.dataset.index || '0', 10);
    return -1;
  }}
  function setActive(chunks, idx) {{
    chunks.forEach(function(c) {{ c.removeAttribute('data-active'); }});
    var target = chunks[idx];
    if (target) {{
      target.setAttribute('data-active', 'true');
      target.focus();
      target.scrollIntoView({{ behavior: 'smooth', block: 'nearest' }});
    }}
  }}
  var pendingG = false;
  document.addEventListener('keydown', function(e) {{
    var chunks = getChunks();
    if (!chunks.length) return;
    var current = getActiveIndex(chunks);
    if (current < 0) current = 0;
    if (e.key === 'j') {{
      e.preventDefault();
      setActive(chunks, Math.min(current + 1, chunks.length - 1));
      pendingG = false;
    }} else if (e.key === 'k') {{
      e.preventDefault();
      setActive(chunks, Math.max(current - 1, 0));
      pendingG = false;
    }} else if (e.key === 'g' && !e.shiftKey) {{
      e.preventDefault();
      if (pendingG) {{ setActive(chunks, 0); pendingG = false; }}
      else {{ pendingG = true; setTimeout(function() {{ pendingG = false; }}, 500); }}
    }} else if (e.key === 'G' || (e.shiftKey && (e.key === 'g' || e.key === 'G'))) {{
      e.preventDefault();
      setActive(chunks, chunks.length - 1);
      pendingG = false;
    }} else if (e.key === 'End') {{
      e.preventDefault();
      setActive(chunks, chunks.length - 1);
      pendingG = false;
    }} else if (e.key === 'c') {{
      e.preventDefault();
      var chunk = chunks[current >= 0 ? current : 0];
      if (chunk) {{ navigator.clipboard.writeText((chunk.textContent || '').trim()).catch(function() {{}}); }}
      pendingG = false;
    }} else if (e.key === 'Escape') {{
      e.preventDefault();
      var a = document.querySelector('[data-active="true"]');
      if (a) a.removeAttribute('data-active');
      pendingG = false;
    }} else {{
      pendingG = false;
    }}
  }});
}})()
        </script>
    "#
    );
    format::html(&page_layout("Browse", "Browse", &body))
}

/// Custom 404 page.
///
/// # Errors
///
/// Returns a 404 HTML response.
pub async fn not_found_page() -> Result<Response> {
    let body = r#"
        <h1>404</h1>
        <p>Not Found</p>
        <p>The page you're looking for doesn't exist.</p>
        <a href="/ui/">Return to Dashboard</a>
    "#;
    format::html(&page_layout("Not Found", "", body))
}

/// Returns the 404 page HTML as a raw string (for use as Axum fallback handler).
#[must_use]
pub fn not_found_html() -> String {
    let body = r#"
        <h1>404</h1>
        <p>Not Found</p>
        <p>The page you're looking for doesn't exist.</p>
        <a href="/ui/">Return to Dashboard</a>
    "#;
    page_layout("Not Found", "", body)
}

/// Registers web UI routes under `/ui`.
#[must_use]
pub fn routes() -> Routes {
    Routes::new()
        .prefix("ui")
        .add("/", get(dashboard))
        .add("/config", get(config_page))
        .add("/health", get(health_page))
        .add("/jobs", get(jobs_page))
        .add("/browse", get(browse_page))
}
