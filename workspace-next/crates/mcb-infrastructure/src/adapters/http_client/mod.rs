//! HTTP Client Infrastructure
//!
//! Internal HTTP client abstraction for embedding and vector store providers.
//! This is NOT a domain port - it's an infrastructure implementation detail
//! that adapters use internally.
//!
//! ## Architecture Note
//!
//! Per Clean Architecture, the domain layer doesn't know about HTTP.
//! This module provides DI-ready HTTP client infrastructure for adapters
//! that need to communicate with external APIs.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use mcb_infrastructure::adapters::http_client::{HttpClientProvider, HttpClientPool};
//! use std::sync::Arc;
//!
//! // Create pool (typically done at startup)
//! let pool = HttpClientPool::new().expect("Failed to create HTTP client pool");
//! let client: Arc<dyn HttpClientProvider> = Arc::new(pool);
//!
//! // Inject into providers
//! // let openai = OpenAIEmbeddingProvider::new(api_key, model, client);
//! ```

mod config;
mod pool;
mod provider;

pub use config::HttpClientConfig;
pub use pool::HttpClientPool;
pub use provider::HttpClientProvider;

/// Type alias for shared HTTP client provider
pub type SharedHttpClient = std::sync::Arc<dyn HttpClientProvider>;

/// Test utilities for HTTP client (null implementations for testing)
pub mod test_utils;
