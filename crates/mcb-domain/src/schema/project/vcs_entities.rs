use super::ForeignKeyDef;
use crate::schema::memory::{IndexDef, TableDef};

pub fn tables() -> Vec<TableDef> {
    vec![
        table!(
            "repositories",
            [
                crate::col!("id", Text, pk),
                crate::col!("org_id", Text),
                crate::col!("project_id", Text),
                crate::col!("name", Text),
                crate::col!("url", Text),
                crate::col!("local_path", Text),
                crate::col!("vcs_type", Text),
                crate::col!("created_at", Integer),
                crate::col!("updated_at", Integer),
            ]
        ),
        table!(
            "branches",
            [
                crate::col!("id", Text, pk),
                crate::col!("repository_id", Text),
                crate::col!("name", Text),
                crate::col!("is_default", Integer),
                crate::col!("head_commit", Text),
                crate::col!("upstream", Text, nullable),
                crate::col!("created_at", Integer),
            ]
        ),
        table!(
            "worktrees",
            [
                crate::col!("id", Text, pk),
                crate::col!("repository_id", Text),
                crate::col!("branch_id", Text),
                crate::col!("path", Text),
                crate::col!("status", Text),
                crate::col!("assigned_agent_id", Text, nullable),
                crate::col!("created_at", Integer),
                crate::col!("updated_at", Integer),
            ]
        ),
        table!(
            "agent_worktree_assignments",
            [
                crate::col!("id", Text, pk),
                crate::col!("agent_session_id", Text),
                crate::col!("worktree_id", Text),
                crate::col!("assigned_at", Integer),
                crate::col!("released_at", Integer, nullable),
            ]
        ),
    ]
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        index!("idx_repositories_org", "repositories", ["org_id"]),
        index!("idx_repositories_project", "repositories", ["project_id"]),
        index!(
            "idx_repositories_url_org",
            "repositories",
            ["org_id", "url"]
        ),
        index!("idx_branches_repo", "branches", ["repository_id"]),
        index!("idx_worktrees_repo", "worktrees", ["repository_id"]),
        index!("idx_worktrees_branch", "worktrees", ["branch_id"]),
        index!("idx_worktrees_agent", "worktrees", ["assigned_agent_id"]),
        index!(
            "idx_agent_worktree_assignments_session",
            "agent_worktree_assignments",
            ["agent_session_id"]
        ),
        index!(
            "idx_agent_worktree_assignments_worktree",
            "agent_worktree_assignments",
            ["worktree_id"]
        ),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        ForeignKeyDef {
            from_table: "repositories".to_string(),
            from_column: "org_id".to_string(),
            to_table: "organizations".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "repositories".to_string(),
            from_column: "project_id".to_string(),
            to_table: "projects".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "branches".to_string(),
            from_column: "repository_id".to_string(),
            to_table: "repositories".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "worktrees".to_string(),
            from_column: "repository_id".to_string(),
            to_table: "repositories".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "worktrees".to_string(),
            from_column: "branch_id".to_string(),
            to_table: "branches".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "agent_worktree_assignments".to_string(),
            from_column: "agent_session_id".to_string(),
            to_table: "agent_sessions".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "agent_worktree_assignments".to_string(),
            from_column: "worktree_id".to_string(),
            to_table: "worktrees".to_string(),
            to_column: "id".to_string(),
        },
    ]
}

pub fn unique_constraints() -> Vec<super::UniqueConstraintDef> {
    vec![
        super::UniqueConstraintDef {
            table: "repositories".to_string(),
            columns: vec![
                "org_id".to_string(),
                "project_id".to_string(),
                "name".to_string(),
            ],
        },
        super::UniqueConstraintDef {
            table: "branches".to_string(),
            columns: vec!["repository_id".to_string(), "name".to_string()],
        },
    ]
}
