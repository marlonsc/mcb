//! Authentication service
//!
//! Handles JWT-based authentication with role-based access control.

use super::claims::{Claims, User};
use super::config::AuthConfig;
use super::password::{migrate_hash, needs_migration, verify_password};
use super::roles::Permission;
use crate::domain::error::{Error, Result};
use crate::infrastructure::utils::RwLockExt;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::sync::RwLock;

/// Authentication and authorization service
///
/// Handles JWT-based authentication with role-based access control.
/// Provides secure user management and permission validation.
///
/// # Security Features
///
/// - JWT token generation and validation
/// - Password-based authentication with Argon2id
/// - Automatic bcrypt to Argon2id migration
/// - Role-based permission checking
/// - Token expiration handling
pub struct AuthService {
    /// Authentication configuration (RwLock for password hash migration)
    config: RwLock<AuthConfig>,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(config: AuthConfig) -> Self {
        // Log security warnings at startup
        config.log_security_warnings();

        Self {
            config: RwLock::new(config),
        }
    }

    /// Create a default auth service with admin user
    pub fn with_default_config() -> Self {
        Self::new(AuthConfig::default())
    }

    /// Validate authentication configuration for production use
    ///
    /// Returns warnings if the configuration uses insecure defaults.
    pub fn validate_config(&self) -> Vec<super::config::SecurityWarning> {
        self.config
            .read()
            .map(|c| c.validate_for_production())
            .unwrap_or_default()
    }

    /// Authenticate user with email and password
    ///
    /// Performs user authentication and returns a JWT token on success.
    /// Automatically migrates bcrypt hashes to Argon2id on successful login.
    pub fn authenticate(&self, email: &str, password: &str) -> Result<String> {
        let config = self.config.read_guard()?;

        if !config.enabled {
            return Err(Error::generic("Authentication is disabled"));
        }

        // Find user by email
        let user = config
            .users
            .get(email)
            .ok_or_else(|| Error::generic("Invalid credentials"))?;

        // Verify password
        if !verify_password(password, &user.password_hash)? {
            return Err(Error::generic("Invalid credentials"));
        }

        // Generate token
        let token = self.generate_token_internal(user, &config)?;

        // Check if hash needs migration (drop read lock first)
        let should_migrate = needs_migration(&user.password_hash);
        let user_email = email.to_string();
        drop(config);

        // Migrate bcrypt hash to Argon2id if needed
        if should_migrate {
            if let Ok(new_hash) = migrate_hash(password) {
                if let Ok(mut config) = self.config.write() {
                    if let Some(user) = config.users.get_mut(&user_email) {
                        user.password_hash = new_hash;
                        user.hash_version = super::claims::HashVersion::Argon2id;
                        tracing::info!(
                            "Migrated password hash for user '{}' from bcrypt to Argon2id",
                            user_email
                        );
                    }
                }
            }
        }

        Ok(token)
    }

    /// Validate JWT token and extract claims
    ///
    /// Parses and validates a JWT token using HMAC-SHA256, checking its signature,
    /// expiration, and extracting the claims payload.
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let config = self.config.read_guard()?;

        if !config.enabled {
            return Err(Error::generic("Authentication is disabled"));
        }

        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.set_issuer(&[&config.jwt_issuer]);

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|e| Error::generic(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Check if user has permission
    pub fn check_permission(&self, claims: &Claims, permission: &Permission) -> bool {
        claims.role.has_permission(permission)
    }

    /// Generate JWT token for user
    fn generate_token_internal(&self, user: &User, config: &AuthConfig) -> Result<String> {
        let claims = Claims::new(
            user.id.clone(),
            user.email.clone(),
            user.role.clone(),
            config.jwt_issuer.clone(),
            config.jwt_expiration,
        );

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
        )
        .map_err(|e| Error::generic(format!("Token generation failed: {}", e)))
    }

    /// Generate a new token for existing claims (refresh)
    pub fn refresh_token(&self, claims: &Claims) -> Result<String> {
        let config = self.config.read_guard()?;

        if !config.enabled {
            return Err(Error::generic("Authentication is disabled"));
        }

        // Verify user still exists
        let user = config
            .users
            .get(&claims.email)
            .ok_or_else(|| Error::generic("User no longer exists"))?;

        // Verify role hasn't been downgraded
        if user.role.level() < claims.role.level() {
            return Err(Error::generic("User role has changed"));
        }

        self.generate_token_internal(user, &config)
    }

    /// Get user by email
    pub fn get_user(&self, email: &str) -> Option<User> {
        self.config
            .read()
            .ok()
            .and_then(|c| c.users.get(email).cloned())
    }

    /// Alias for get_user for semantic clarity
    pub fn get_user_by_email(&self, email: &str) -> Option<User> {
        self.get_user(email)
    }

    /// Check if authentication is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.read().map(|c| c.enabled).unwrap_or(false)
    }

    /// Check if a path should bypass authentication
    pub fn should_bypass(&self, path: &str) -> bool {
        self.config
            .read()
            .map(|c| c.should_bypass(path))
            .unwrap_or(false)
    }

    /// Get a copy of the current configuration
    pub fn config(&self) -> Option<AuthConfig> {
        self.config.read().ok().map(|c| c.clone())
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::with_default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::auth::roles::UserRole;

    fn test_config() -> AuthConfig {
        let mut config =
            AuthConfig::new("test-secret-at-least-32-bytes-long".to_string(), 3600, true);

        // Add test user with known password
        let hash = super::super::password::hash_password("testpass").unwrap();
        config.add_user(User::new(
            "test".to_string(),
            "test@example.com".to_string(),
            UserRole::Developer,
            hash,
        ));

        config
    }

    #[test]
    fn test_authenticate_success() {
        let service = AuthService::new(test_config());

        let token = service.authenticate("test@example.com", "testpass");
        assert!(token.is_ok(), "Authentication should succeed");
    }

    #[test]
    fn test_authenticate_wrong_password() {
        let service = AuthService::new(test_config());

        let result = service.authenticate("test@example.com", "wrongpass");
        assert!(result.is_err(), "Authentication should fail");
    }

    #[test]
    fn test_authenticate_unknown_user() {
        let service = AuthService::new(test_config());

        let result = service.authenticate("unknown@example.com", "testpass");
        assert!(result.is_err(), "Authentication should fail");
    }

    #[test]
    fn test_validate_token() {
        let service = AuthService::new(test_config());

        let token = service
            .authenticate("test@example.com", "testpass")
            .unwrap();
        let claims = service.validate_token(&token);

        assert!(claims.is_ok());
        let claims = claims.unwrap();
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.role, UserRole::Developer);
    }

    #[test]
    fn test_disabled_auth() {
        let mut config = test_config();
        config.enabled = false;

        let service = AuthService::new(config);

        assert!(!service.is_enabled());
        assert!(service
            .authenticate("test@example.com", "testpass")
            .is_err());
    }

    #[test]
    fn test_check_permission() {
        let service = AuthService::new(test_config());

        let token = service
            .authenticate("test@example.com", "testpass")
            .unwrap();
        let claims = service.validate_token(&token).unwrap();

        // Developer should have indexing permission
        assert!(service.check_permission(&claims, &Permission::IndexCodebase));
        // Developer should NOT have user management permission
        assert!(!service.check_permission(&claims, &Permission::ManageUsers));
    }

    #[test]
    fn test_bypass_paths() {
        let service = AuthService::new(test_config());

        assert!(service.should_bypass("/api/health"));
        assert!(!service.should_bypass("/api/search"));
    }
}
