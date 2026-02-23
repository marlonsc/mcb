//! GraphQL playground and handler endpoints for Seaography auto-API.

use async_graphql::{
    dynamic::Schema,
    http::{GraphQLPlaygroundConfig, playground_source},
};
use async_graphql_axum::GraphQLRequest;
use loco_rs::prelude::*;
use seaography::async_graphql;

async fn graphql_playground() -> Result<Response> {
    let config =
        GraphQLPlaygroundConfig::new("/api/graphql").with_header("Authorization", "AUTO_TOKEN");

    let res = playground_source(config).replace(
        r#""Authorization":"AUTO_TOKEN""#,
        r#""Authorization":`Bearer ${localStorage.getItem('auth_token')}`"#,
    );

    Ok(Response::new(res.into()))
}

async fn graphql_handler(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    gql_req: GraphQLRequest,
) -> std::result::Result<async_graphql_axum::GraphQLResponse, (axum::http::StatusCode, &'static str)>
{
    let _user_email = auth.claims.pid;

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
