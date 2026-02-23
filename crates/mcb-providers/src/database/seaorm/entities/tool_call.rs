//! SeaORM entity for the `tool_calls` table.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tool_calls")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub org_id: Option<String>,
    pub project_id: Option<String>,
    pub repo_id: Option<String>,
    pub session_id: String,
    pub tool_name: String,
    pub params_summary: Option<String>,
    pub success: i64,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::agent_session::Entity",
        from = "Column::SessionId",
        to = "super::agent_session::Column::Id"
    )]
    AgentSession,
}

impl Related<super::agent_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AgentSession.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
