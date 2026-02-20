//!
//! **Documentation**: [docs/modules/server.md](../../../docs/modules/server.md)
//!
//! Authentication and Authorization
//!
//! Handles user authentication and authorization for admin interfaces.
//! Uses infrastructure auth services through dependency injection.

/// Authentication handler for admin interfaces
pub struct AuthHandler;

impl Default for AuthHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthHandler {
    /// Create a new auth handler
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}
