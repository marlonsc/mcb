//! Dependency Graph Validation
//!
//! Validates Clean Architecture layer boundaries:
//! - domain: No internal dependencies (pure domain entities)
//! - application: Only domain (use cases and ports)
//! - providers: domain and application (adapter implementations)
//! - infrastructure: domain, application, and providers (DI composition root)
//! - server: domain, application, and infrastructure (transport layer)
//! - mcb: All crates (facade that re-exports entire public API)

mod bypass;
mod cargo;
mod cycles;
mod uses;
mod validator;
mod violation;

pub use self::validator::DependencyValidator;
pub use self::violation::{DependencyCycle, DependencyViolation};
