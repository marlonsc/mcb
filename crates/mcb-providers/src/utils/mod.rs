//! Provider Utilities
//!
//! Shared utilities used by provider implementations.

pub(crate) mod http;
mod http_response;
mod json;

pub(crate) use http::{handle_request_error, parse_embedding_vector};
pub use http_response::HttpResponseUtils;
pub use json::JsonExt;
