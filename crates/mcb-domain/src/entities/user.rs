//! User entity — a human or service account within an organization.

use serde::{Deserialize, Serialize};

/// A user belongs to exactly one organization and can be a member of
/// multiple teams. Users authenticate via API keys (Phase 1) and
/// external IdP / OAuth in later phases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier (UUID).
    pub id: String,
    /// Organization this user belongs to (tenant isolation).
    pub org_id: String,
    /// Email address (unique within an org).
    pub email: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Role within the organization (e.g. "admin", "member", "viewer").
    pub role: UserRole,
    /// Bcrypt/Argon2 hash of the user's primary API key (nullable — set on first key creation).
    pub api_key_hash: Option<String>,
    /// Timestamp when the user was created (Unix epoch).
    pub created_at: i64,
    /// Timestamp when the user was last updated (Unix epoch).
    pub updated_at: i64,
}

/// Role a user holds within an organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    /// Full administrative access.
    Admin,
    /// Standard member with read/write access.
    Member,
    /// Read-only viewer.
    Viewer,
    /// Service account (API-only, used by agents).
    Service,
}

impl UserRole {
    /// Returns the string representation of the user role.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Viewer => "viewer",
            Self::Service => "service",
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Self::Admin),
            "member" => Ok(Self::Member),
            "viewer" => Ok(Self::Viewer),
            "service" => Ok(Self::Service),
            _ => Err(format!("Unknown user role: {s}")),
        }
    }
}
