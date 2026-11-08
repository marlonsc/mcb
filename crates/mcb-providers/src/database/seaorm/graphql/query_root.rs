//! Seaography auto-generated GraphQL schema for all MCB entities.

use seaography::{Builder, BuilderContext, LifecycleHooks, MultiLifecycleHooks, async_graphql};

use async_graphql::dynamic::{Schema, SchemaError};
use sea_orm::DatabaseConnection;

use std::any::Any;
use std::sync::Arc;

use mcb_domain::ports::GraphQLSchemaProvider;
use mcb_domain::registry::graphql::GraphQLSchemaProviderConfig;

/// Builds the Seaography GraphQL schema wiring all MCB entity modules.
///
/// The schema is intended to be built once at startup and stored in
/// [`loco_rs::app::AppContext::shared_store`].
///
/// # Errors
///
/// Returns [`SchemaError`] if the GraphQL schema fails to build.
pub fn schema(
    context: &'static BuilderContext,
    database: DatabaseConnection,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> Result<Schema, SchemaError> {
    let builder = Builder::new(context, database.clone());
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

/// Seaography GraphQL schema provider implementing the domain port.
///
/// The [`BuilderContext`] is owned by this provider instance (injected at
/// construction) instead of a process-wide `lazy_static`. seaography's
/// `Builder::new` requires a `&'static BuilderContext`, so the owned context
/// is pinned once per provider — one provider instance resolves per process
/// through the registry factory.
struct SeaographyGraphQLSchemaProvider {
    context: &'static BuilderContext,
}

impl SeaographyGraphQLSchemaProvider {
    fn new() -> Self {
        let context = Box::leak(Box::new(BuilderContext {
            hooks: LifecycleHooks::new(MultiLifecycleHooks::default()),
            ..Default::default()
        }));
        Self { context }
    }
}

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
        let s = schema(self.context, *database, depth, complexity).map_err(|e| {
            mcb_domain::error::Error::configuration(format!("GraphQL schema build failed: {e}"))
        })?;
        Ok(Box::new(s))
    }
}

/// Factory function for creating the Seaography GraphQL provider.
fn seaography_factory(
    _config: &GraphQLSchemaProviderConfig,
) -> mcb_domain::error::Result<Arc<dyn GraphQLSchemaProvider>> {
    Ok(Arc::new(SeaographyGraphQLSchemaProvider::new()))
}

mcb_domain::register_graphql_schema_provider!(
    "seaography",
    "Seaography auto-generated GraphQL schema from SeaORM entities",
    seaography_factory
);
