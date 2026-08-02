//! Configuration page — shows current MCB configuration.

use super::html_page;
use crate::controllers::admin_config::load_admin_config;
use crate::state::McbState;
use axum::extract::Extension;
use loco_rs::prelude::*;

/// Helper to extract a nested JSON string or return "—".
fn cfg_str<'a>(config: &'a serde_json::Value, path: &[&str]) -> &'a str {
    let mut v = config;
    for key in path {
        v = match v.get(*key) {
            Some(next) => next,
            None => return "—",
        };
    }
    v.as_str().unwrap_or("—")
}

/// Helper to extract a nested JSON i64 or return "—".
fn cfg_i64(config: &serde_json::Value, path: &[&str]) -> String {
    let mut v = config;
    for key in path {
        v = match v.get(*key) {
            Some(next) => next,
            None => return "—".to_owned(),
        };
    }
    v.as_i64().map_or_else(|| "—".to_owned(), |n| n.to_string())
}

/// Configuration page handler.
///
/// # Errors
///
/// Fails when config cannot be loaded.
pub async fn config_page(Extension(state): Extension<McbState>) -> Result<Response> {
    let provider = state.embedding_provider.provider_name();
    let dims = state.embedding_provider.dimensions();
    let cfg = load_admin_config()?;

    let body = format!(
        r#"<h1>Configuration</h1>
<div class="config-grid">
  <div class="card"><h3>Embedding Provider</h3>
    <table class="data-table">
      <tr><td>Provider</td><td><strong>{provider}</strong></td></tr>
      <tr><td>Dimensions</td><td>{dims}</td></tr>
    </table>
  </div>
  <div class="card"><h3>Vector Store</h3>
    <table class="data-table">
      <tr><td>Provider</td><td><strong>{vp}</strong></td></tr>
      <tr><td>Host</td><td>{vh}</td></tr>
      <tr><td>Port</td><td>{vport}</td></tr>
    </table>
  </div>
  <div class="card"><h3>Database</h3>
    <table class="data-table">
      <tr><td>URI</td><td>{db}</td></tr>
    </table>
  </div>
</div>"#,
        vp = cfg_str(&cfg, &["providers", "vector_store", "provider"]),
        vh = cfg_str(&cfg, &["providers", "vector_store", "host"]),
        vport = cfg_i64(&cfg, &["providers", "vector_store", "port"]),
        db = cfg_str(&cfg, &["database", "uri"]),
    );
    html_page!("Configuration", body)
}
