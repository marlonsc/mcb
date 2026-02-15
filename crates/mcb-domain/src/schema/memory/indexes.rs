use super::columns::COL_OBSERVATION_TYPE;

/// Index definition.
#[derive(Debug, Clone)]
pub struct IndexDef {
    /// Index name.
    pub name: String,
    /// Table name this index is on.
    pub table: String,
    /// Columns included in this index.
    pub columns: Vec<String>,
}

/// Returns index definitions for the memory module.
pub fn indexes() -> Vec<IndexDef> {
    vec![
        IndexDef {
            name: "idx_obs_hash".to_string(),
            table: "observations".to_string(),
            columns: vec!["content_hash".to_string()],
        },
        IndexDef {
            name: "idx_obs_created".to_string(),
            table: "observations".to_string(),
            columns: vec!["created_at".to_string()],
        },
        IndexDef {
            name: "idx_obs_type".to_string(),
            table: "observations".to_string(),
            columns: vec![COL_OBSERVATION_TYPE.to_string()],
        },
        IndexDef {
            name: "idx_obs_embedding".to_string(),
            table: "observations".to_string(),
            columns: vec!["embedding_id".to_string()],
        },
        IndexDef {
            name: "idx_summary_session".to_string(),
            table: "session_summaries".to_string(),
            columns: vec!["session_id".to_string()],
        },
    ]
}
