//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
/// Default HTTP request timeout in seconds
pub const DEFAULT_HTTP_TIMEOUT_SECS: u64 = 30;

/// HTTP request timeout error message template
pub const ERROR_MSG_REQUEST_TIMEOUT: &str = "Request timed out after {:?}";

/// HTTP Authorization header name.
pub const HTTP_HEADER_AUTHORIZATION: &str = "Authorization";

/// HTTP Content-Type header name.
pub const HTTP_HEADER_CONTENT_TYPE: &str = "Content-Type";

/// Pinecone API key header name.
pub const PINECONE_API_KEY_HEADER: &str = "Api-Key";
