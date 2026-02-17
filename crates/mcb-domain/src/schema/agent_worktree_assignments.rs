use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "agent_worktree_assignments",
        [
            crate::col!("id", Text, pk),
            crate::col!("agent_session_id", Text),
            crate::col!("worktree_id", Text),
            crate::col!("assigned_at", Integer),
            crate::col!("released_at", Integer, nullable),
            crate::col!("origin_context", Text, nullable),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!(
            "idx_agent_worktree_assignments_session",
            "agent_worktree_assignments",
            ["agent_session_id"]
        ),
        crate::index!(
            "idx_agent_worktree_assignments_worktree",
            "agent_worktree_assignments",
            ["worktree_id"]
        ),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        crate::fk!(
            "agent_worktree_assignments",
            "agent_session_id",
            "agent_sessions",
            "id"
        ),
        crate::fk!(
            "agent_worktree_assignments",
            "worktree_id",
            "worktrees",
            "id"
        ),
    ]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
