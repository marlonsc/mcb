//! Authentication repository ports.

use async_trait::async_trait;

use crate::entities::User;
use crate::error::Result;

/// User information with API key details.
#[derive(Debug, Clone)]
pub struct UserWithApiKey {
    /// The user entity.
    pub user: User,
    /// API key ID.
    pub api_key_id: String,
    /// API key hash.
    pub api_key_hash: String,
}

/// API key information for validation.
#[derive(Debug, Clone)]
pub struct ApiKeyInfo {
    /// API key ID.
    pub api_key_id: String,
    /// User ID associated with the key.
    pub user_id: String,
    /// Organization ID (if applicable).
    pub organization_id: Option<String>,
}

/// Port for authentication repository operations.
#[async_trait]
pub trait AuthRepositoryPort: Send + Sync {
    /// Find users by API key hash.
    async fn find_users_by_api_key_hash(&self, key_hash: &str) -> Result<Vec<UserWithApiKey>>;
    /// Verify API key and return key info if valid.
    async fn verify_api_key(&self, key_hash: &str) -> Result<Option<ApiKeyInfo>>;
}
