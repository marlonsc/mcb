//! SeaORM entity for the `project_issues` table.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "project_issues")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub org_id: String,
    pub project_id: String,
    pub phase_id: Option<String>,
    pub title: String,
    pub description: String,
    pub issue_type: String,
    pub status: String,
    pub priority: i64,
    pub assignee: Option<String>,
    pub labels: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub closed_at: Option<i64>,
    pub created_by: String,
    pub estimated_minutes: Option<i64>,
    pub actual_minutes: Option<i64>,
    pub notes: String,
    pub design: String,
    pub parent_issue_id: Option<String>,
    pub closed_reason: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::organization::Entity",
        from = "Column::OrgId",
        to = "super::organization::Column::Id"
    )]
    Organization,
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id"
    )]
    Project,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::CreatedBy",
        to = "super::user::Column::Id"
    )]
    Creator,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentIssueId",
        to = "Column::Id"
    )]
    ParentIssue,
    #[sea_orm(has_many = "super::issue_comment::Entity")]
    Comments,
    #[sea_orm(has_many = "super::issue_label_assignment::Entity")]
    LabelAssignments,
}

impl Related<super::organization::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Organization.def()
    }
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::issue_comment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Comments.def()
    }
}

impl Related<super::issue_label_assignment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LabelAssignments.def()
    }
}

impl Related<super::issue_label::Entity> for Entity {
    fn to() -> RelationDef {
        super::issue_label_assignment::Relation::IssueLabel.def()
    }
    fn via() -> Option<RelationDef> {
        Some(
            super::issue_label_assignment::Relation::ProjectIssue
                .def()
                .rev(),
        )
    }
}

impl ActiveModelBehavior for ActiveModel {}
