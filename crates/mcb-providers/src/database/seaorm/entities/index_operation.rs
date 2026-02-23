//! SeaORM entity for the `index_operations` table.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "index_operations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub collection_id: String,
    pub status: String,
    pub total_files: i64,
    pub processed_files: i64,
    pub current_file: Option<String>,
    pub error_message: Option<String>,
    pub started_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
