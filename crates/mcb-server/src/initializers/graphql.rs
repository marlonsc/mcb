//! GraphQL schema initializer — builds the Seaography schema at startup.

use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::prelude::*;

const DEPTH: Option<usize> = Some(100);
const COMPLEXITY: Option<usize> = Some(1000);

/// Loco initializer that builds the Seaography GraphQL schema and stores it
/// in [`AppContext::shared_store`] for use by the GraphQL controller.
pub struct GraphQLInitializer;

#[async_trait]
impl Initializer for GraphQLInitializer {
    fn name(&self) -> String {
        "graphql".to_owned()
    }

    async fn after_routes(&self, router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        let schema = mcb_providers::database::seaorm::graphql::query_root::schema(
            ctx.db.clone(),
            DEPTH,
            COMPLEXITY,
        )
        .map_err(|e| loco_rs::Error::string(&format!("GraphQL schema build failed: {e}")))?;
        ctx.shared_store.insert(schema);

        Ok(router)
    }
}
