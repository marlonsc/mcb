//! `SeaORM` database layer: entities, migrations, repositories, and GraphQL.
/// `SeaORM` adapter for `AuthRepositoryPort` (API key verification, user lookup by key hash).
pub mod auth_repository;
/// SeaQuery constraint builder for vector search filtering.
pub mod constraints;
pub mod conversions;
/// `SeaORM` adapter for `DashboardQueryPort` (observations, tool calls, agent session stats).
pub mod dashboard;
/// Generated SeaORM entity modules (`sea-orm-codegen`).
pub mod entities;
/// Seaography-backed GraphQL schema and query root.
pub mod graphql;
pub mod migration;
pub mod repos;

pub use dashboard::SeaOrmDashboardAdapter;

pub use auth_repository::SeaOrmAuthRepositoryAdapter;
