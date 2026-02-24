//! `SeaORM` repository implementations.
//!
//! This module groups concrete persistence repositories used by the server and
//! infrastructure layers.

/// Agent repository implementation.
pub mod agent;
/// Generic entity repository implementation.
pub mod entity;
/// Indexing repository implementation.
pub mod index;
/// Observation repository implementation.
pub mod observation;
/// Project repository implementation.
pub mod project;
/// Agent session repository implementation.
pub mod session;
/// VCS repository implementation.
pub mod vcs;

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
/// `SeaORM` agent session repository.
pub use session::SeaOrmAgentSessionRepository;
/// `SeaORM` VCS repository.
pub use vcs::SeaOrmVcsEntityRepository;
