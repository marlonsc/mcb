//! SeaORM entity for the `checkpoints` table.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "checkpoints")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub session_id: String,
    pub checkpoint_type: String,
    pub description: String,
    pub snapshot_data: String,
    pub created_at: i64,
    pub restored_at: Option<i64>,
    pub expired: Option<i64>,
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
