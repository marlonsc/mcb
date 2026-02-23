//! SeaORM entity for the `file_hashes` table.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "file_hashes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub project_id: String,
    pub collection: String,
    pub file_path: String,
    pub content_hash: String,
    pub indexed_at: i64,
    pub deleted_at: Option<i64>,
    pub origin_context: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id"
    )]
    Project,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
