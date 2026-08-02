//! Health page — shows provider health status.

use super::{html_page, status_badge};
use crate::state::McbState;
use axum::extract::Extension;
use loco_rs::prelude::*;

/// Health page handler.
///
/// # Errors
///
/// Fails when health checks cannot be performed.
pub async fn health_page(Extension(state): Extension<McbState>) -> Result<Response> {
    let emb_ok = state.embedding_provider.health_check().await.is_ok();
    let vec_ok = state.vector_store.health_check().await.is_ok();
    let overall = if emb_ok && vec_ok {
        "healthy"
    } else {
        "degraded"
    };

    let badge = |ok: bool| {
        if ok {
            status_badge!("healthy", "ok")
        } else {
            status_badge!("degraded", "error")
        }
    };

    let body = format!(
        r#"<h1>Health</h1>
<div class="health-status">
  <h2>Status: {}</h2>
  <div class="health-cards">
    <div class="card"><h3>Embedding Provider</h3><p>{}</p>
      <p class="muted">Provider: {}</p>
      <p class="muted">Dimensions: {}</p>
    </div>
    <div class="card"><h3>Vector Store</h3><p>{}</p></div>
  </div>
</div>"#,
        status_badge!(overall, overall),
        badge(emb_ok),
        state.embedding_provider.provider_name(),
        state.embedding_provider.dimensions(),
        badge(vec_ok),
    );
    html_page!("Health", body)
}
