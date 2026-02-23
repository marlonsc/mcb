//! SeaORM entity for the `users` table.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub org_id: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub api_key_hash: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::organization::Entity",
        from = "Column::OrgId",
        to = "super::organization::Column::Id"
    )]
    Organization,
    #[sea_orm(has_many = "super::api_key::Entity")]
    ApiKeys,
}

impl Related<super::organization::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Organization.def()
    }
}

impl Related<super::api_key::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ApiKeys.def()
    }
}

impl Related<super::team::Entity> for Entity {
    fn to() -> RelationDef {
        super::team_member::Relation::Team.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::team_member::Relation::User.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
