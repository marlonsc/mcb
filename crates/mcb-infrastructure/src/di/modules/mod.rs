//! DI Module Organization - Simple Container Pattern
//!
//! This module provides domain service factories for runtime service creation.
//! External providers (embedding, `vector_store`, cache, language) are resolved
//! dynamically via the registry system in `di/resolver.rs`.
//!
//! Infrastructure and admin services are created directly in `di/bootstrap.rs`.

/// Domain services factory
pub mod domain_services;

// Re-export domain services types
pub use domain_services::{DomainServicesContainer, DomainServicesFactory, ServiceDependencies};
