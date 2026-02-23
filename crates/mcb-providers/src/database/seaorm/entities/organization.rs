//! SeaORM entity for the `organizations` table.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "organizations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    #[sea_orm(unique)]
    pub slug: String,
    pub settings_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user::Entity")]
    Users,
    #[sea_orm(has_many = "super::team::Entity")]
    Teams,
    #[sea_orm(has_many = "super::api_key::Entity")]
    ApiKeys,
    #[sea_orm(has_many = "super::project::Entity")]
    Projects,
    #[sea_orm(has_many = "super::plan::Entity")]
    Plans,
    #[sea_orm(has_many = "super::plan_version::Entity")]
    PlanVersions,
    #[sea_orm(has_many = "super::plan_review::Entity")]
    PlanReviews,
    #[sea_orm(has_many = "super::repository::Entity")]
    Repositories,
    #[sea_orm(has_many = "super::project_issue::Entity")]
    ProjectIssues,
    #[sea_orm(has_many = "super::issue_label::Entity")]
    IssueLabels,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::team::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Teams.def()
    }
}

impl Related<super::api_key::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ApiKeys.def()
    }
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
}

impl Related<super::plan::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plans.def()
    }
}

impl Related<super::plan_version::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlanVersions.def()
    }
}

impl Related<super::plan_review::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlanReviews.def()
    }
}

impl Related<super::repository::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Repositories.def()
    }
}

impl Related<super::project_issue::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectIssues.def()
    }
}

impl Related<super::issue_label::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::IssueLabels.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
