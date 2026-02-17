use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "worktrees",
        [
            crate::col!("id", Text, pk),
            crate::col!("org_id", Text),
            crate::col!("project_id", Text),
            crate::col!("repository_id", Text),
            crate::col!("branch_id", Text),
            crate::col!("path", Text),
            crate::col!("status", Text),
            crate::col!("assigned_agent_id", Text, nullable),
            crate::col!("origin_context", Text, nullable),
            crate::col!("created_at", Integer),
            crate::col!("updated_at", Integer),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!("idx_worktrees_repo", "worktrees", ["repository_id"]),
        crate::index!("idx_worktrees_branch", "worktrees", ["branch_id"]),
        crate::index!("idx_worktrees_agent", "worktrees", ["assigned_agent_id"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        crate::fk!("worktrees", "repository_id", "repositories", "id"),
        crate::fk!("worktrees", "branch_id", "branches", "id"),
    ]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
