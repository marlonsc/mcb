//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Authentication constants — Single Source of Truth

/// Default HTTP header name for API key authentication.
pub const API_KEY_HEADER: &str = "x-api-key";
/// Prefix for bearer token authentication in the Authorization header.
pub const BEARER_PREFIX: &str = "Bearer ";
/// Constant value for `JWT_DEFAULT_EXPIRATION_SECS`.
pub const JWT_DEFAULT_EXPIRATION_SECS: u64 = 86400;
/// Constant value for `JWT_REFRESH_EXPIRATION_SECS`.
pub const JWT_REFRESH_EXPIRATION_SECS: u64 = 604800;
/// Constant value for `BCRYPT_DEFAULT_COST`.
pub const BCRYPT_DEFAULT_COST: u32 = 12;
/// Constant value for `DEFAULT_ADMIN_KEY_HEADER`.
pub const DEFAULT_ADMIN_KEY_HEADER: &str = "X-Admin-Key";
/// Default Authorization header name.
pub use crate::constants::http::HTTP_HEADER_AUTHORIZATION as AUTHORIZATION_HEADER;
/// Constant value for `MIN_JWT_SECRET_LENGTH`.
pub const MIN_JWT_SECRET_LENGTH: usize = 32;
