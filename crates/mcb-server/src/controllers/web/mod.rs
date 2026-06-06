//! Web controllers for MCB admin HTML pages served at `/ui/*`.
//!
//! Each page is a submodule. Shared HTML helpers are macros to avoid repetition.

mod browse;
mod config;
mod dashboard;
mod health;
mod jobs;

pub use browse::browse_page;
pub use config::config_page;
pub use dashboard::dashboard;
pub use health::health_page;
pub use jobs::jobs_page;

use loco_rs::prelude::*;

// ---------------------------------------------------------------------------
// HTML generation macros (crate-local, used by page submodules)
// ---------------------------------------------------------------------------

/// Wrap body HTML in the page layout and return an Axum `Response`.
macro_rules! html_page {
    ($title:expr, $body:expr) => {
        loco_rs::prelude::format::html(&super::page_layout($title, $title, &$body))
    };
}

/// Render a dashboard metric card.
macro_rules! metric_card {
    ($title:expr, $value:expr, $label:expr) => {
        format!(
            r#"<div class="card metric-card"><h3>{}</h3><div class="metric-value">{}</div><div class="metric-label">{}</div></div>"#,
            $title, $value, $label
        )
    };
}

/// Render a status badge `<span>`.
macro_rules! status_badge {
    ($class:expr, $text:expr) => {
        format!(r#"<span class="status-badge {}">{}</span>"#, $class, $text)
    };
}

/// Render a `<table class="data-table">` from header + rows, or a muted fallback.
macro_rules! data_table {
    ($headers:expr, $rows:expr, $empty_msg:expr) => {
        if $rows.is_empty() {
            format!(r#"<p class="muted">{}</p>"#, $empty_msg)
        } else {
            format!(
                r#"<table class="data-table"><thead><tr>{}</tr></thead><tbody>{}</tbody></table>"#,
                $headers, $rows
            )
        }
    };
}

pub(crate) use {data_table, html_page, metric_card, status_badge};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Common HTML page layout with nav and theme toggle.
fn page_layout(title: &str, active: &str, body: &str) -> String {
    let nav = |href: &str, label: &str| -> String {
        let cls = if label == active {
            " class=\"active\""
        } else {
            ""
        };
        format!(r#"<a href="{href}"{cls}>{label}</a>"#)
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
  <script>(function(){{ var s=localStorage.getItem('mcb-theme'); if(s) document.documentElement.setAttribute('data-theme',s); }})()</script>
</head>
<body>
  <nav class="main-nav">
    <div class="nav-brand">MCB Admin</div>
    <div class="nav-links">{d}{c}{h}{j}{b}</div>
    <button title="Toggle Theme" aria-label="Toggle theme" onclick="toggleTheme()">🌓</button>
  </nav>
  <main class="content">{body}</main>
  <script src="/ui/shared.js"></script>
  <script>function toggleTheme(){{ var h=document.documentElement,c=h.getAttribute('data-theme')||'light',n=c==='light'?'dark':c==='dark'?'auto':'light'; h.setAttribute('data-theme',n); localStorage.setItem('mcb-theme',n); }}</script>
</body>
</html>"#,
        d = nav("/ui/", "Dashboard"),
        c = nav("/ui/config", "Configuration"),
        h = nav("/ui/health", "Health"),
        j = nav("/ui/jobs", "Jobs"),
        b = nav("/ui/browse", "Browse"),
    )
}

/// Escape HTML special characters.
pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Custom 404 page.
///
/// # Errors
///
/// Returns a 404 HTML response.
pub async fn not_found_page() -> Result<Response> {
    let body = r#"<h1>404</h1><p>Not Found</p><a href="/ui/">Return to Dashboard</a>"#;
    format::html(&page_layout("Not Found", "", body))
}

/// 404 HTML as raw string (for Axum fallback handler).
#[must_use]
pub fn not_found_html() -> String {
    let body = r#"<h1>404</h1><p>Not Found</p><a href="/ui/">Return to Dashboard</a>"#;
    page_layout("Not Found", "", body)
}

/// Register web UI routes under `/ui`.
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
