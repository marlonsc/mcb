/// Constant value for `JWT_DEFAULT_EXPIRATION_SECS`.
pub const JWT_DEFAULT_EXPIRATION_SECS: u64 = 86400;
/// Constant value for `JWT_REFRESH_EXPIRATION_SECS`.
pub const JWT_REFRESH_EXPIRATION_SECS: u64 = 604800;
/// Constant value for `BCRYPT_DEFAULT_COST`.
pub const BCRYPT_DEFAULT_COST: u32 = 12;
/// Constant value for `API_KEY_HEADER`.
pub const API_KEY_HEADER: &str = "x-api-key";
/// Constant value for `DEFAULT_ADMIN_KEY_HEADER`.
pub const DEFAULT_ADMIN_KEY_HEADER: &str = "X-Admin-Key";
/// Constant value for `AUTHORIZATION_HEADER`.
pub const AUTHORIZATION_HEADER: &str = "authorization";
/// Constant value for `BEARER_PREFIX`.
pub const BEARER_PREFIX: &str = "Bearer ";
/// Constant value for `MIN_JWT_SECRET_LENGTH`.
pub const MIN_JWT_SECRET_LENGTH: usize = 32;
