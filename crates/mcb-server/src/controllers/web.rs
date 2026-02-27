//! Web controllers for custom MCB admin HTML pages served at `/ui/*`.

use crate::state::McbState;
use axum::extract::Extension;
use loco_rs::prelude::*;
use mcb_domain::ports::{IndexingOperationStatus, ValidationStatus};

/// Common HTML page layout wrapper.
fn page_layout(title: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en" data-theme="light">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MCB Admin - {title}</title>
    <link rel="icon" href="/favicon.ico" type="image/svg+xml">
    <link rel="stylesheet" href="/ui/theme.css">
</head>
<body>
    <nav class="main-nav">
        <div class="nav-brand">MCB Admin</div>
        <div class="nav-links">
            <a href="/ui/">Dashboard</a>
            <a href="/ui/config">Configuration</a>
            <a href="/ui/health">Health</a>
            <a href="/ui/jobs">Jobs</a>
            <a href="/ui/browse">Browse</a>
        </div>
        <button title="Theme Toggle" aria-label="Toggle theme" onclick="toggleTheme()">🌓</button>
    </nav>
    <main class="content">
        {body}
    </main>
    <script src="/ui/shared.js"></script>
    <script>
        function toggleTheme() {{
            const html = document.documentElement;
            const current = html.getAttribute('data-theme') || 'light';
            html.setAttribute('data-theme', current === 'light' ? 'dark' : 'light');
        }}
    </script>
</body>
</html>"#
    )
}

/// Dashboard page — main admin landing page.
///
/// # Errors
///
/// Fails when dashboard data cannot be loaded.
pub async fn dashboard(Extension(_state): Extension<McbState>) -> Result<Response> {
    let body = r#"
        <h1>Dashboard</h1>
        <div class="dashboard-grid">
            <div class="card">
                <h3>System Overview</h3>
                <p>MCB Memory Context Browser</p>
            </div>
        </div>
    "#;
    format::html(&page_layout("Dashboard", body))
}

/// Configuration page — shows current MCB configuration.
///
/// # Errors
///
/// Fails when config cannot be loaded.
pub async fn config_page(Extension(state): Extension<McbState>) -> Result<Response> {
    let provider_name = state.embedding_provider.provider_name();
    let dimensions = state.embedding_provider.dimensions();

    let body = format!(
        r#"
        <h1>Configuration</h1>
        <div class="config-section">
            <h3>Embedding Provider</h3>
            <table>
                <tr><td>Provider</td><td>{provider_name}</td></tr>
                <tr><td>Dimensions</td><td>{dimensions}</td></tr>
            </table>
            <h3>Vector Store</h3>
            <table>
                <tr><td>Host</td><td>localhost</td></tr>
                <tr><td>Port</td><td>19530</td></tr>
            </table>
        </div>
    "#
    );
    format::html(&page_layout("Configuration", &body))
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
    let embedding_status = if embedding_ok { "ok" } else { "error" };
    let vector_status = if vector_ok { "ok" } else { "error" };

    let body = format!(
        r#"
        <h1>Health</h1>
        <div class="health-status">
            <h2>Status: <span class="status-badge">{overall}</span></h2>
            <div class="health-cards">
                <div class="card">
                    <h3>Embedding Provider</h3>
                    <p>Status: {embedding_status}</p>
                    <p>Provider: {provider}</p>
                </div>
                <div class="card">
                    <h3>Vector Store</h3>
                    <p>Status: {vector_status}</p>
                </div>
            </div>
        </div>
    "#,
        provider = state.embedding_provider.provider_name(),
    );
    format::html(&page_layout("Health", &body))
}

/// Jobs page — shows indexing and validation operations.
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

    let body = format!(
        r#"
        <h1>Jobs</h1>
        <div class="jobs-summary">
            <h2>Indexing Operations</h2>
            <p>Status: <span class="status-badge">{status}</span></p>
            <div class="stats">
                <span>Total: {total}</span>
                <span>Running: {running}</span>
                <span>Complete: {complete}</span>
            </div>
        </div>
    "#,
        complete = total - running,
    );
    format::html(&page_layout("Jobs", &body))
}

/// Browse page — shows vector store collections.
///
/// # Errors
///
/// Fails when collection data cannot be loaded.
pub async fn browse_page(Extension(state): Extension<McbState>) -> Result<Response> {
    let collections = state
        .vector_store
        .list_collections()
        .await
        .unwrap_or_default();

    let items_html: String = if collections.is_empty() {
        "<p>No collections found.</p>".to_owned()
    } else {
        collections
            .iter()
            .map(|c| format!(r#"<div class="card"><h3>{}</h3></div>"#, c.name))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let body = format!(
        r#"
        <h1>Browse</h1>
        <div id="collections-grid" class="collections-grid">
            {items_html}
        </div>
    "#
    );
    format::html(&page_layout("Browse", &body))
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
    format::html(&page_layout("Not Found", body))
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
    page_layout("Not Found", body)
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
