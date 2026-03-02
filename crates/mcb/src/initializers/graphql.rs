//! # GraphQL Schema Initializer
//!
//! Builds the GraphQL schema at startup via CA/DI resolution through
//! [`mcb_domain::registry::graphql`]. No direct dependency on providers.

use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::prelude::*;

use mcb_server::constants::graphql::{SCHEMA_COMPLEXITY, SCHEMA_DEPTH};

/// Loco initializer that builds the GraphQL schema and stores it
/// in [`AppContext::shared_store`] for use by the GraphQL controller.
pub struct GraphQLInitializer;

#[async_trait]
impl Initializer for GraphQLInitializer {
    fn name(&self) -> String {
        "graphql".to_owned()
    }

    async fn after_routes(&self, router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        // Resolve the GraphQL schema provider via domain DI registry
        let provider = mcb_domain::registry::graphql::resolve_graphql_schema_provider(
            &mcb_domain::registry::graphql::GraphQLSchemaProviderConfig::new("seaography"),
        )
        .map_err(|e| loco_rs::Error::string(&e.to_string()))?;

        // Pass the database connection as type-erased Any
        let db_any: Box<dyn std::any::Any + Send + Sync> = Box::new(ctx.db.clone());
        let schema_any = provider
            .build_schema(db_any, Some(SCHEMA_DEPTH), Some(SCHEMA_COMPLEXITY))
            .map_err(|e| loco_rs::Error::string(&format!("GraphQL schema build failed: {e}")))?;

        // Insert the schema as-is into shared_store. The GraphQL controller in
        // mcb-server knows the concrete type and will downcast when needed.
        mcb_server::graphql_store::insert_schema(&ctx.shared_store, schema_any)
            .map_err(|e| loco_rs::Error::string(&e))?;

        Ok(router)
    }
}
