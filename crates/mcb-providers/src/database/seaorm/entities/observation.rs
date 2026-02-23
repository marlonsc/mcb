//! SeaORM entity for the `observations` table.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "observations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub project_id: String,
    pub content: String,
    #[sea_orm(unique)]
    pub content_hash: String,
    pub tags: Option<String>,
    pub observation_type: Option<String>,
    pub metadata: Option<String>,
    pub created_at: i64,
    pub embedding_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id"
    )]
    Project,
    #[sea_orm(has_many = "super::error_pattern_match::Entity")]
    ErrorPatternMatches,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::error_pattern_match::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ErrorPatternMatches.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
