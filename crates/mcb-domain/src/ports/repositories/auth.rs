//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#repository-ports)
//!
//! Authentication Repository Port
//!
//! # Overview
//! Defines the interface for authentication-related repository operations,
//! including API key validation and user lookup by API key.

use async_trait::async_trait;

use crate::entities::User;
use crate::error::Result;

/// User information with API key details
#[derive(Debug, Clone)]
pub struct UserWithApiKey {
    /// The user entity
    pub user: User,
    /// API key ID
    pub api_key_id: String,
    /// API key hash
    pub api_key_hash: String,
}

/// API key information for validation
#[derive(Debug, Clone)]
pub struct ApiKeyInfo {
    /// API key ID
    pub api_key_id: String,
    /// User ID associated with the key
    pub user_id: String,
    /// Organization ID (if applicable)
    pub organization_id: Option<String>,
}

/// Port for authentication repository operations
#[async_trait]
pub trait AuthRepositoryPort: Send + Sync {
    /// Find users by API key hash
    ///
    /// # Arguments
    /// * `key_hash` - The hash of the API key to search for
    ///
    /// # Returns
    /// A vector of users with their associated API key information
    async fn find_users_by_api_key_hash(&self, key_hash: &str) -> Result<Vec<UserWithApiKey>>;

    /// Verify API key and return key info if valid
    ///
    /// # Arguments
    /// * `key_hash` - The hash of the API key to verify
    ///
    /// # Returns
    /// API key information if the key is valid, None otherwise
    async fn verify_api_key(&self, key_hash: &str) -> Result<Option<ApiKeyInfo>>;
}
