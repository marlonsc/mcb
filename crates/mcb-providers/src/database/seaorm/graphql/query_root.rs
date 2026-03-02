//! Seaography auto-generated GraphQL schema for all MCB entities.

use seaography::{
    Builder, BuilderContext, LifecycleHooks, MultiLifecycleHooks, async_graphql, lazy_static,
};

use async_graphql::dynamic::{Schema, SchemaError};
use sea_orm::DatabaseConnection;

lazy_static::lazy_static! {
    static ref CONTEXT: BuilderContext = {
        BuilderContext {
            hooks: LifecycleHooks::new(MultiLifecycleHooks::default()),
            ..Default::default()
        }
    };
}

/// Builds the Seaography GraphQL schema wiring all MCB entity modules.
///
/// The schema is intended to be built once at startup and stored in
/// [`loco_rs::app::AppContext::shared_store`].
///
/// # Errors
///
/// Returns [`SchemaError`] if the GraphQL schema fails to build.
pub fn schema(
    database: DatabaseConnection,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> Result<Schema, SchemaError> {
    let builder = Builder::new(&CONTEXT, database.clone());
    let builder = crate::database::seaorm::entities::register_entity_modules(builder);
    builder
        .set_depth_limit(depth)
        .set_complexity_limit(complexity)
        .schema_builder()
        .data(database)
        .finish()
}

// ============================================================================
// CA/DI: GraphQLSchemaProvider port implementation + linkme registration
// ============================================================================

use std::any::Any;
use std::sync::Arc;

use mcb_domain::ports::GraphQLSchemaProvider;
use mcb_domain::registry::graphql::{
    GRAPHQL_SCHEMA_PROVIDERS, GraphQLSchemaProviderConfig, GraphQLSchemaProviderEntry,
};

/// Seaography GraphQL schema provider implementing the domain port.
struct SeaographyGraphQLSchemaProvider;

impl GraphQLSchemaProvider for SeaographyGraphQLSchemaProvider {
    fn build_schema(
        &self,
        db: Box<dyn Any + Send + Sync>,
        depth: Option<usize>,
        complexity: Option<usize>,
    ) -> mcb_domain::error::Result<Box<dyn Any + Send + Sync>> {
        let database = db.downcast::<DatabaseConnection>().map_err(|_| {
            mcb_domain::error::Error::configuration(
                "GraphQL: expected DatabaseConnection, got wrong type".to_owned(),
            )
        })?;
        let s = schema(*database, depth, complexity).map_err(|e| {
            mcb_domain::error::Error::configuration(format!("GraphQL schema build failed: {e}"))
        })?;
        Ok(Box::new(s))
    }
}

/// Factory function for creating the Seaography GraphQL provider.
fn seaography_factory(
    _config: &GraphQLSchemaProviderConfig,
) -> std::result::Result<Arc<dyn GraphQLSchemaProvider>, String> {
    Ok(Arc::new(SeaographyGraphQLSchemaProvider))
}

#[linkme::distributed_slice(GRAPHQL_SCHEMA_PROVIDERS)]
static SEAOGRAPHY_GRAPHQL_PROVIDER: GraphQLSchemaProviderEntry = GraphQLSchemaProviderEntry {
    name: "seaography",
    description: "Seaography auto-generated GraphQL schema from SeaORM entities",
    build: seaography_factory,
};
