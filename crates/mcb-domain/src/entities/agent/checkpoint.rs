//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#core-entities)
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::types::CheckpointType;

/// Represents a saved state of an agent session that can be restored.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Checkpoint {
    /// Unique identifier for the checkpoint.
    pub id: String,
    /// Session identifier this checkpoint belongs to.
    pub session_id: String,
    /// Type of checkpoint (e.g., Automatic, Manual).
    pub checkpoint_type: CheckpointType,
    /// Human-readable description of what this checkpoint represents.
    pub description: String,
    /// Serialized snapshot of the agent's state.
    pub snapshot_data: serde_json::Value,
    /// Unix timestamp when the checkpoint was created.
    pub created_at: i64,
    /// Unix timestamp of the last time this checkpoint was restored.
    pub restored_at: Option<i64>,
    /// Whether this checkpoint has expired and is eligible for cleanup.
    pub expired: bool,
}
