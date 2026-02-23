//! ApiKey entity — bearer tokens for authenticating users and agents.
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#core-entities)
//!

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

crate::define_entity! {
    /// An API key is a bearer credential scoped to a user within an
    /// organization. Keys can be narrowed by JSON-encoded scopes and
    /// optionally expire. Revocation is tracked via `revoked_at`.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct ApiKey { id, org_id, created_at } {
        /// User identifier this key is associated with.
        pub user_id: String,
        /// Hashed representation of the API key.
        pub key_hash: String,
        /// Readable name for the API key.
        pub name: String,
        /// JSON-encoded list of allowed scopes.
        pub scopes_json: String,
        /// Optional expiration timestamp.
        pub expires_at: Option<i64>,
        /// Optional revocation timestamp.
        pub revoked_at: Option<i64>,
    }
}

crate::impl_table_schema!(ApiKey, "api_keys",
    columns: [
        ("id", Text, pk),
        ("user_id", Text),
        ("org_id", Text),
        ("key_hash", Text),
        ("name", Text),
        ("scopes_json", Text),
        ("expires_at", Integer, nullable),
        ("created_at", Integer),
        ("revoked_at", Integer, nullable),
    ],
    indexes: [
        "idx_api_keys_user" => ["user_id"],
        "idx_api_keys_org" => ["org_id"],
        "idx_api_keys_key_hash" => ["key_hash"],
    ],
    foreign_keys: [
        ("user_id", "users", "id"),
        ("org_id", "organizations", "id"),
    ],
    unique_constraints: [],
);
