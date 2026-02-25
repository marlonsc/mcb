//! # CA Exception: Seaography GraphQL Initializer
//!
//! This initializer is a declared Clean Architecture exception.
//! Seaography auto-generates GraphQL schema from `SeaORM` entities,
//! requiring direct `DatabaseConnection` access via `ctx.db`.
//! See docs/architecture/ARCHITECTURE.md for rationale.
//!
//! GraphQL schema initializer — builds the Seaography schema at startup.

use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::prelude::*;

use crate::constants::graphql::{SCHEMA_COMPLEXITY, SCHEMA_DEPTH};

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
            Some(SCHEMA_DEPTH),
            Some(SCHEMA_COMPLEXITY),
        )
        .map_err(|e| loco_rs::Error::string(&format!("GraphQL schema build failed: {e}")))?;
        ctx.shared_store.insert(schema);

        Ok(router)
    }
}
