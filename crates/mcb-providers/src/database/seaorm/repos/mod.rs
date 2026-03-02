//! `SeaORM` repository implementations.
//!
//! This module groups concrete persistence repositories used by the server and
//! infrastructure layers.

/// Agent repository implementation.
pub mod agent;
mod common;
/// Generic entity repository implementation.
pub mod entity;
/// Indexing repository implementation.
pub mod index;
/// Observation repository implementation.
pub mod observation;
/// Project repository implementation.
pub mod project;
/// Database repository bundle registry integration.
pub mod registry;

/// `SeaORM` agent repository.
pub use agent::SeaOrmAgentRepository;
/// `SeaORM` generic entity repository.
pub use entity::SeaOrmEntityRepository;
/// `SeaORM` indexing repository.
pub use index::SeaOrmIndexRepository;
/// `SeaORM` observation repository.
pub use observation::SeaOrmObservationRepository;
/// `SeaORM` project repository.
pub use project::SeaOrmProjectRepository;
