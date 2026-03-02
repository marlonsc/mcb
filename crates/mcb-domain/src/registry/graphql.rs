//! GraphQL schema provider registry.
//!
//! Auto-registration for GraphQL schema builders via linkme.

use std::collections::HashMap;

/// Configuration for GraphQL schema provider resolution.
#[derive(Debug, Clone, Default)]
pub struct GraphQLSchemaProviderConfig {
    /// Provider name (e.g. "seaography").
    pub provider: String,
    /// Additional provider-specific configuration.
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(GraphQLSchemaProviderConfig {});

crate::impl_registry!(
    provider_trait: crate::ports::GraphQLSchemaProvider,
    config_type: GraphQLSchemaProviderConfig,
    entry_type: GraphQLSchemaProviderEntry,
    slice_name: GRAPHQL_SCHEMA_PROVIDERS,
    resolve_fn: resolve_graphql_schema_provider,
    list_fn: list_graphql_schema_providers
);
