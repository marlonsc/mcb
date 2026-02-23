//! SeaORM entity for the `delegations` table.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "delegations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub parent_session_id: String,
    pub child_session_id: String,
    pub prompt: String,
    pub prompt_embedding_id: Option<String>,
    pub result: Option<String>,
    pub success: i64,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub duration_ms: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::agent_session::Entity",
        from = "Column::ParentSessionId",
        to = "super::agent_session::Column::Id"
    )]
    ParentSession,
    #[sea_orm(
        belongs_to = "super::agent_session::Entity",
        from = "Column::ChildSessionId",
        to = "super::agent_session::Column::Id"
    )]
    ChildSession,
}

impl Related<super::agent_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ParentSession.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
