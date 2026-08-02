//! GraphQL schema provider ports.

use std::sync::Arc;

/// Domain port for GraphQL schema providers.
///
/// Implementations build a GraphQL schema from the database connection
/// and return it as a type-erased `Box<dyn Any + Send + Sync>`.
/// The concrete schema type (e.g. `async_graphql::dynamic::Schema`) lives
/// in the provider layer; the domain remains free of GraphQL dependencies.
pub trait GraphQLSchemaProvider: Send + Sync {
    /// Build a GraphQL schema using the given opaque database connection.
    ///
    /// # Arguments
    /// * `db` - Database connection as `Box<dyn Any + Send + Sync>`
    /// * `depth` - Optional query depth limit
    /// * `complexity` - Optional query complexity limit
    ///
    /// # Errors
    /// Returns an error if the schema build fails.
    fn build_schema(
        &self,
        db: Box<dyn std::any::Any + Send + Sync>,
        depth: Option<usize>,
        complexity: Option<usize>,
    ) -> crate::error::Result<Box<dyn std::any::Any + Send + Sync>>;
}

/// Shared GraphQL schema provider for dependency injection.
pub type SharedGraphQLSchemaProvider = Arc<dyn GraphQLSchemaProvider>;
