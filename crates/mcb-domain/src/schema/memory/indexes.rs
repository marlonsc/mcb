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
#[must_use]
pub fn indexes() -> Vec<IndexDef> {
    vec![
        IndexDef {
            name: "idx_obs_hash".to_owned(),
            table: "observations".to_owned(),
            columns: vec!["content_hash".to_owned()],
        },
        IndexDef {
            name: "idx_obs_created".to_owned(),
            table: "observations".to_owned(),
            columns: vec!["created_at".to_owned()],
        },
        IndexDef {
            name: "idx_obs_type".to_owned(),
            table: "observations".to_owned(),
            columns: vec![COL_OBSERVATION_TYPE.to_owned()],
        },
        IndexDef {
            name: "idx_obs_embedding".to_owned(),
            table: "observations".to_owned(),
            columns: vec!["embedding_id".to_owned()],
        },
        IndexDef {
            name: "idx_summary_session".to_owned(),
            table: "session_summaries".to_owned(),
            columns: vec!["session_id".to_owned()],
        },
    ]
}
