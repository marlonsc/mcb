//! SeaORM entity for the `projects` table.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub path: String,
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
    #[sea_orm(has_many = "super::collection::Entity")]
    Collections,
    #[sea_orm(has_many = "super::observation::Entity")]
    Observations,
    #[sea_orm(has_many = "super::session_summary::Entity")]
    SessionSummaries,
    #[sea_orm(has_many = "super::file_hash::Entity")]
    FileHashes,
    #[sea_orm(has_many = "super::error_pattern::Entity")]
    ErrorPatterns,
    #[sea_orm(has_many = "super::project_issue::Entity")]
    ProjectIssues,
    #[sea_orm(has_many = "super::plan::Entity")]
    Plans,
    #[sea_orm(has_many = "super::repository::Entity")]
    Repositories,
    #[sea_orm(has_many = "super::issue_label::Entity")]
    IssueLabels,
}

impl Related<super::organization::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Organization.def()
    }
}

impl Related<super::collection::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Collections.def()
    }
}

impl Related<super::observation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Observations.def()
    }
}

impl Related<super::session_summary::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SessionSummaries.def()
    }
}

impl Related<super::file_hash::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FileHashes.def()
    }
}

impl Related<super::error_pattern::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ErrorPatterns.def()
    }
}

impl Related<super::project_issue::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectIssues.def()
    }
}

impl Related<super::plan::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plans.def()
    }
}

impl Related<super::repository::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Repositories.def()
    }
}

impl Related<super::issue_label::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::IssueLabels.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
