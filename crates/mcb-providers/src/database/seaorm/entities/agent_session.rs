//! SeaORM entity for the `agent_sessions` table.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "agent_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub project_id: Option<String>,
    pub worktree_id: Option<String>,
    pub session_summary_id: String,
    pub agent_type: String,
    pub model: String,
    pub parent_session_id: Option<String>,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub duration_ms: Option<i64>,
    pub status: String,
    pub prompt_summary: Option<String>,
    pub result_summary: Option<String>,
    pub token_count: Option<i64>,
    pub tool_calls_count: Option<i64>,
    pub delegations_count: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentSessionId",
        to = "Column::Id"
    )]
    ParentSession,
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id"
    )]
    Project,
    #[sea_orm(
        belongs_to = "super::worktree::Entity",
        from = "Column::WorktreeId",
        to = "super::worktree::Column::Id"
    )]
    Worktree,
    #[sea_orm(has_many = "super::delegation::Entity")]
    DelegationsAsParent,
    #[sea_orm(has_many = "super::tool_call::Entity")]
    ToolCalls,
    #[sea_orm(has_many = "super::checkpoint::Entity")]
    Checkpoints,
    #[sea_orm(has_many = "super::agent_worktree_assignment::Entity")]
    WorktreeAssignments,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::worktree::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Worktree.def()
    }
}

impl Related<super::tool_call::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ToolCalls.def()
    }
}

impl Related<super::checkpoint::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Checkpoints.def()
    }
}

impl Related<super::agent_worktree_assignment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WorktreeAssignments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
