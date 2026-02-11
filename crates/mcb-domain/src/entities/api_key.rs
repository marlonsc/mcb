//! ApiKey entity — bearer tokens for authenticating users and agents.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// An API key is a bearer credential scoped to a user within an
/// organization. Keys can be narrowed by JSON-encoded scopes and
/// optionally expire. Revocation is tracked via `revoked_at`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiKey {
    /// Unique identifier (UUID).
    pub id: String,
    /// User who owns this key.
    pub user_id: String,
    /// Organization this key belongs to (denormalized for fast lookup).
    pub org_id: String,
    /// Bcrypt/Argon2 hash of the raw key material (never store plaintext).
    pub key_hash: String,
    /// Human-readable name for the key (e.g. "CI pipeline", "dev laptop").
    pub name: String,
    /// JSON array of scope strings describing allowed operations
    /// (e.g. `["read:code","write:memory"]`). Empty `"[]"` = full access.
    pub scopes_json: String,
    /// Optional expiration timestamp (Unix epoch). `None` = never expires.
    pub expires_at: Option<i64>,
    /// Timestamp when the key was created (Unix epoch).
    pub created_at: i64,
    /// Timestamp when the key was revoked (Unix epoch). `None` = active.
    pub revoked_at: Option<i64>,
}
