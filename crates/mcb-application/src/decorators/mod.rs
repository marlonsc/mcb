//! Decorator Module - SOLID Open/Closed Compliant
//!
//! **Documentation**: [docs/modules/application.md](../../../../docs/modules/application.md#decorators)
//!
//! Provides decorators that wrap providers to add cross-cutting concerns
//! without modifying the original provider implementations.
//!
//! ## Design Pattern
//!
//! All decorators follow the Decorator Pattern to comply with SOLID Open/Closed:
//! - Wraps an existing provider via `Arc<dyn Trait>`
//! - Implements the same trait as the wrapped provider
//! - Adds behavior (metrics, logging, caching) without modification
//! - Can be stacked: `Instrumented(Cached(Provider))`
