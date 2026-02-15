//! Decorator Module - SOLID Open/Closed Compliant
//!
//! Provides decorators that wrap providers to add cross-cutting concerns
//! without modifying the original provider implementations.
//!
//! ## Available Decorators
//!
//! | Decorator | Purpose |
//! | ----------- | --------- |
//! | [`InstrumentedEmbeddingProvider`] | Adds timing metrics to embedding operations |
//!
//! ## Design Pattern
//!
//! All decorators follow the Decorator Pattern to comply with SOLID Open/Closed:
//! - Wraps an existing provider via `Arc<dyn Trait>`
//! - Implements the same trait as the wrapped provider
//! - Adds behavior (metrics, logging, caching) without modification
//! - Can be stacked: `Instrumented(Cached(Provider))`

mod instrumented_embedding;

pub use instrumented_embedding::InstrumentedEmbeddingProvider;
