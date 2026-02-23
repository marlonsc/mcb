//! SeaORM entity for the `issue_labels` table.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "issue_labels")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub org_id: String,
    pub project_id: String,
    pub name: String,
    pub color: String,
    pub created_at: i64,
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

impl Related<super::issue_label_assignment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LabelAssignments.def()
    }
}

impl Related<super::project_issue::Entity> for Entity {
    fn to() -> RelationDef {
        super::issue_label_assignment::Relation::ProjectIssue.def()
    }
    fn via() -> Option<RelationDef> {
        Some(
            super::issue_label_assignment::Relation::IssueLabel
                .def()
                .rev(),
        )
    }
}

impl ActiveModelBehavior for ActiveModel {}
