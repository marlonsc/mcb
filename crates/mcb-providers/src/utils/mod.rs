//! Provider Utilities
//!
//! Shared utilities used by provider implementations.

pub(crate) mod http;
pub(crate) mod http_response;
mod json;

pub(crate) use http::parse_embedding_vector;
pub use json::JsonExt;
