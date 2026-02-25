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
