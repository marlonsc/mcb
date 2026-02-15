//! Multi-tenant schema elements: organizations, users, teams, `api_keys`.

use super::ForeignKeyDef;
use crate::schema::memory::{IndexDef, TableDef};

/// Performs the tables operation.
#[must_use]
pub fn tables() -> Vec<TableDef> {
    vec![
        table!(
            "organizations",
            [
                crate::col!("id", Text, pk),
                crate::col!("name", Text),
                crate::col!("slug", Text, unique),
                crate::col!("settings_json", Text),
                crate::col!("created_at", Integer),
                crate::col!("updated_at", Integer),
            ]
        ),
        table!(
            "users",
            [
                crate::col!("id", Text, pk),
                crate::col!("org_id", Text),
                crate::col!("email", Text),
                crate::col!("display_name", Text),
                crate::col!("role", Text),
                crate::col!("api_key_hash", Text, nullable),
                crate::col!("created_at", Integer),
                crate::col!("updated_at", Integer),
            ]
        ),
        table!(
            "teams",
            [
                crate::col!("id", Text, pk),
                crate::col!("org_id", Text),
                crate::col!("name", Text),
                crate::col!("created_at", Integer),
            ]
        ),
        table!(
            "team_members",
            [
                crate::col!("team_id", Text, pk),
                crate::col!("user_id", Text, pk),
                crate::col!("role", Text),
                crate::col!("joined_at", Integer),
            ]
        ),
        table!(
            "api_keys",
            [
                crate::col!("id", Text, pk),
                crate::col!("user_id", Text),
                crate::col!("org_id", Text),
                crate::col!("key_hash", Text),
                crate::col!("name", Text),
                crate::col!("scopes_json", Text),
                crate::col!("expires_at", Integer, nullable),
                crate::col!("created_at", Integer),
                crate::col!("revoked_at", Integer, nullable),
            ]
        ),
    ]
}

/// Performs the indexes operation.
#[must_use]
pub fn indexes() -> Vec<IndexDef> {
    vec![
        index!("idx_users_org", "users", ["org_id"]),
        index!("idx_users_email", "users", ["email"]),
        index!("idx_users_api_key_hash", "users", ["api_key_hash"]),
        index!("idx_teams_org", "teams", ["org_id"]),
        index!("idx_team_members_team", "team_members", ["team_id"]),
        index!("idx_team_members_user", "team_members", ["user_id"]),
        index!("idx_api_keys_user", "api_keys", ["user_id"]),
        index!("idx_api_keys_org", "api_keys", ["org_id"]),
        index!("idx_api_keys_key_hash", "api_keys", ["key_hash"]),
        index!("idx_organizations_name", "organizations", ["name"]),
    ]
}

/// Performs the foreign keys operation.
#[must_use]
pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        ForeignKeyDef {
            from_table: "users".to_owned(),
            from_column: "org_id".to_owned(),
            to_table: "organizations".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "teams".to_owned(),
            from_column: "org_id".to_owned(),
            to_table: "organizations".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "team_members".to_owned(),
            from_column: "team_id".to_owned(),
            to_table: "teams".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "team_members".to_owned(),
            from_column: "user_id".to_owned(),
            to_table: "users".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "api_keys".to_owned(),
            from_column: "user_id".to_owned(),
            to_table: "users".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "api_keys".to_owned(),
            from_column: "org_id".to_owned(),
            to_table: "organizations".to_owned(),
            to_column: "id".to_owned(),
        },
        // projects.org_id → organizations.id (upgrade existing FK)
        ForeignKeyDef {
            from_table: "projects".to_owned(),
            from_column: "org_id".to_owned(),
            to_table: "organizations".to_owned(),
            to_column: "id".to_owned(),
        },
    ]
}

/// Performs the unique constraints operation.
#[must_use]
pub fn unique_constraints() -> Vec<super::UniqueConstraintDef> {
    vec![
        super::UniqueConstraintDef {
            table: "users".to_owned(),
            columns: vec!["org_id".to_owned(), "email".to_owned()],
        },
        super::UniqueConstraintDef {
            table: "teams".to_owned(),
            columns: vec!["org_id".to_owned(), "name".to_owned()],
        },
        super::UniqueConstraintDef {
            table: "team_members".to_owned(),
            columns: vec!["team_id".to_owned(), "user_id".to_owned()],
        },
    ]
}
