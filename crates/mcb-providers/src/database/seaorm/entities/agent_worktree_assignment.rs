//! SeaORM entity for the `agent_worktree_assignments` table.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "agent_worktree_assignments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub agent_session_id: String,
    pub worktree_id: String,
    pub assigned_at: i64,
    pub released_at: Option<i64>,
    pub origin_context: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::agent_session::Entity",
        from = "Column::AgentSessionId",
        to = "super::agent_session::Column::Id"
    )]
    AgentSession,
    #[sea_orm(
        belongs_to = "super::worktree::Entity",
        from = "Column::WorktreeId",
        to = "super::worktree::Column::Id"
    )]
    Worktree,
}

impl Related<super::agent_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AgentSession.def()
    }
}

impl Related<super::worktree::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Worktree.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
