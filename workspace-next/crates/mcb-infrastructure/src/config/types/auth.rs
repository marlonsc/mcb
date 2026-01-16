//! Authentication configuration types

use crate::constants::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Password hashing algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PasswordAlgorithm {
    Argon2, // Argon2id (recommended)
    Bcrypt, // bcrypt
    Pbkdf2, // PBKDF2
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication
    pub enabled: bool,

    /// JWT secret key
    pub jwt_secret: String,

    /// JWT expiration time in seconds
    pub jwt_expiration_secs: u64,

    /// JWT refresh token expiration in seconds
    pub jwt_refresh_expiration_secs: u64,

    /// API key authentication enabled
    pub api_key_enabled: bool,

    /// API key header name
    pub api_key_header: String,

    /// User database path
    pub user_db_path: Option<PathBuf>,

    /// Password hashing algorithm
    pub password_algorithm: PasswordAlgorithm,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            jwt_secret: crate::crypto::TokenGenerator::generate_secure_token(32),
            jwt_expiration_secs: JWT_DEFAULT_EXPIRATION_SECS,
            jwt_refresh_expiration_secs: JWT_REFRESH_EXPIRATION_SECS,
            api_key_enabled: true,
            api_key_header: API_KEY_HEADER.to_string(),
            user_db_path: None,
            password_algorithm: PasswordAlgorithm::Argon2,
        }
    }
}