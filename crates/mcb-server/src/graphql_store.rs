//! Thin helper for inserting a type-erased GraphQL schema into Loco's shared store.
//!
//! This keeps the composition root (`mcb` crate) free of `async-graphql`
//! dependencies while allowing the DI-resolved schema to be correctly stored.

use std::any::Any;

use async_graphql::dynamic::Schema;

/// Insert a type-erased GraphQL schema into Loco's `SharedStore`.
///
/// The `schema_any` value must be a `Box<async_graphql::dynamic::Schema>` produced
/// by a [`GraphQLSchemaProvider`](mcb_domain::ports::GraphQLSchemaProvider).
///
/// # Errors
///
/// Returns an error string if the downcast fails.
pub fn insert_schema(
    store: &loco_rs::app::SharedStore,
    schema_any: Box<dyn Any + Send + Sync>,
) -> std::result::Result<(), String> {
    let schema = schema_any
        .downcast::<Schema>()
        .map_err(|_| "GraphQL: expected async_graphql::dynamic::Schema, got wrong type")?;
    store.insert(*schema);
    Ok(())
}
