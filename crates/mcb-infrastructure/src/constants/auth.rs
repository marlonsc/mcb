pub const JWT_DEFAULT_EXPIRATION_SECS: u64 = 86400;
pub const JWT_REFRESH_EXPIRATION_SECS: u64 = 604800;
pub const BCRYPT_DEFAULT_COST: u32 = 12;
pub const API_KEY_HEADER: &str = "x-api-key";
pub const DEFAULT_ADMIN_KEY_HEADER: &str = "X-Admin-Key";
pub const AUTHORIZATION_HEADER: &str = "authorization";
pub const BEARER_PREFIX: &str = "Bearer ";
pub const MIN_JWT_SECRET_LENGTH: usize = 32;
