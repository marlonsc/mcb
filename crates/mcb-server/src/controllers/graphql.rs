//! # CA Exception: Seaography GraphQL
//!
//! This controller is a declared Clean Architecture exception.
//! Seaography auto-generates GraphQL schema from `SeaORM` entities,
//! requiring direct `DatabaseConnection` access via `ctx.db`.
//! See docs/architecture/ARCHITECTURE.md for rationale.
//!
//! GraphQL playground and handler endpoints for Seaography auto-API.

use async_graphql::{
    dynamic::Schema,
    http::{GraphQLPlaygroundConfig, playground_source},
};
use async_graphql_axum::GraphQLRequest;
use axum::{extract::Extension, http::HeaderMap};
use loco_rs::prelude::*;
use seaography::async_graphql;

use crate::state::McbState;

// Loco handlers must return the large loco_rs::Error; suppress here.

/// Sentinel the playground embeds so its client script can swap the real key
/// from `localStorage` at render time. It is a placeholder token, not a
/// credential: the server authorizes every request against the stored API
/// keys, so the literal value grants nothing. Both occurrences MUST stay
/// byte-identical for the render-time replacement to match.
const PLAYGROUND_API_KEY_SENTINEL: &str = "AUTO_KEY";

// Loco handlers must return the large loco_rs::Error; suppress here.
#[allow(clippy::result_large_err)]
async fn graphql_playground() -> Result<Response> {
    let config = GraphQLPlaygroundConfig::new("/api/graphql")
        .with_header("X-API-Key", PLAYGROUND_API_KEY_SENTINEL);

    let res = playground_source(config).replace(
        format!(r#""X-API-Key":"{PLAYGROUND_API_KEY_SENTINEL}""#).as_str(),
        r#""X-API-Key":`${localStorage.getItem('api_key') || ''}`"#,
    );

    Ok(Response::new(res.into()))
}

async fn graphql_handler(
    Extension(state): Extension<McbState>,
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    gql_req: GraphQLRequest,
) -> std::result::Result<async_graphql_axum::GraphQLResponse, (axum::http::StatusCode, &'static str)>
{
    crate::auth::authorize_admin_api_key(
        state.auth_repo.as_ref(),
        &headers,
        ctx.config.settings.as_ref(),
    )
    .await
    .map_err(|_| (axum::http::StatusCode::UNAUTHORIZED, "Unauthorized"))?;

    let mut gql_req = gql_req.into_inner();
    gql_req = gql_req.data(seaography::UserContext { user_id: 0 });

    let schema: Schema = ctx.shared_store.get().ok_or((
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "GraphQL not setup",
    ))?;
    let res: async_graphql_axum::GraphQLResponse = schema.execute(gql_req).await.into();

    Ok(res)
}

/// Registers GraphQL playground (`GET /graphql`) and handler (`POST /graphql`) routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("graphql")
        .add("/", get(graphql_playground))
        .add("/", post(graphql_handler))
}
