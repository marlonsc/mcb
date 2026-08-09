//!
//! Infrastructure Services
//!
//! Infrastructure service implementations for port traits defined in mcb-domain.
//! Concrete types are composed in the DI bootstrap module or `loco_app.rs`.

pub mod events;
pub mod indexing;

/// DI-resolved database migrator (CA pattern via domain registry).
pub mod migration;
pub mod readiness;
pub mod validation_ops;
pub mod validator_job_runner;

pub use indexing::DefaultIndexingOperations;
pub use migration::DynamicMigrator;
pub use readiness::RuntimeReadiness;
pub use validation_ops::DefaultValidationOperations;
pub use validator_job_runner::DefaultValidatorJobRunner;
