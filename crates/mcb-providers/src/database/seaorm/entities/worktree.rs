//! SeaORM entity for the `worktrees` table.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "worktrees")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub org_id: Option<String>,
    pub project_id: Option<String>,
    pub repository_id: String,
    pub branch_id: String,
    pub path: String,
    pub status: String,
    pub assigned_agent_id: Option<String>,
    pub origin_context: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::repository::Entity",
        from = "Column::RepositoryId",
        to = "super::repository::Column::Id"
    )]
    Repository,
    #[sea_orm(
        belongs_to = "super::branch::Entity",
        from = "Column::BranchId",
        to = "super::branch::Column::Id"
    )]
    Branch,
    #[sea_orm(has_many = "super::agent_worktree_assignment::Entity")]
    AgentAssignments,
    #[sea_orm(has_many = "super::agent_session::Entity")]
    AgentSessions,
}

impl Related<super::repository::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Repository.def()
    }
}

impl Related<super::branch::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Branch.def()
    }
}

impl Related<super::agent_worktree_assignment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AgentAssignments.def()
    }
}

impl Related<super::agent_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AgentSessions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
