use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Common metadata for domain entities.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct EntityMetadata {
    /// Unique identifier (UUID).
    pub id: String,
    /// Creation timestamp (Unix epoch).
    pub created_at: i64,
    /// Last update timestamp (Unix epoch).
    pub updated_at: i64,
}

/// Trait for entities that have standard metadata.
pub trait BaseEntity {
    /// Returns the entity's unique identifier.
    fn id(&self) -> &str;
    /// Returns the creation timestamp.
    fn created_at(&self) -> i64;
    /// Returns the last update timestamp.
    fn updated_at(&self) -> i64;
}
