use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "team_members",
        [
            crate::col!("team_id", Text, pk),
            crate::col!("user_id", Text, pk),
            crate::col!("role", Text),
            crate::col!("joined_at", Integer),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!("idx_team_members_team", "team_members", ["team_id"]),
        crate::index!("idx_team_members_user", "team_members", ["user_id"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        crate::fk!("team_members", "team_id", "teams", "id"),
        crate::fk!("team_members", "user_id", "users", "id"),
    ]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    vec![crate::unique!("team_members", ["team_id", "user_id"])]
}
