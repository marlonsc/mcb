use serde::{Deserialize, Serialize};

use super::types::CheckpointType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub session_id: String,
    pub checkpoint_type: CheckpointType,
    pub description: String,
    pub snapshot_data: serde_json::Value,
    pub created_at: i64,
    pub restored_at: Option<i64>,
    pub expired: bool,
}
