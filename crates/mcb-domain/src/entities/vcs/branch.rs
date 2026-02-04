use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcsBranch {
    pub id: String,
    pub name: String,
    pub head_commit: String,
    pub is_default: bool,
    pub upstream: Option<String>,
}
