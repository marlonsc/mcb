//! SeaORM database layer — entities, migrations, repositories, and GraphQL.
pub mod conversions;
/// Generated SeaORM entity modules (`sea-orm-codegen`).
pub mod entities;
/// Seaography-backed GraphQL schema and query root.
pub mod graphql;
pub mod migration;
pub mod repos;
