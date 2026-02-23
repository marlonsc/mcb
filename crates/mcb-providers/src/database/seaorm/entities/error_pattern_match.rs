//! SeaORM entity for the `error_pattern_matches` table.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "error_pattern_matches")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub pattern_id: String,
    pub observation_id: String,
    pub confidence: i64,
    pub solution_applied: Option<i64>,
    pub resolution_successful: Option<i64>,
    pub matched_at: i64,
    pub resolved_at: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::error_pattern::Entity",
        from = "Column::PatternId",
        to = "super::error_pattern::Column::Id"
    )]
    ErrorPattern,
    #[sea_orm(
        belongs_to = "super::observation::Entity",
        from = "Column::ObservationId",
        to = "super::observation::Column::Id"
    )]
    Observation,
}

impl Related<super::error_pattern::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ErrorPattern.def()
    }
}

impl Related<super::observation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Observation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
