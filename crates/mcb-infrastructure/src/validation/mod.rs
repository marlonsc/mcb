//! Validation Infrastructure Module
//!
//! Provides the implementation of `ValidationServiceInterface` that
//! delegates to mcb-validate for architecture validation.

mod service;

pub use service::InfraValidationService;
