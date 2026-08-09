//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Provider Utilities
//!
//! Shared utilities used by provider implementations.

/// Shared embedding provider utilities (HTTP client, batch processing, parsing).
pub mod embedding;
/// HTTP request utilities for provider implementations.
pub mod http;
pub(crate) mod http_response;
/// Vector store shared utilities.
pub mod vector_store;
