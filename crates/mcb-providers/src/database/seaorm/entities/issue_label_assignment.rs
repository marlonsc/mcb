//! SeaORM entity for the `issue_label_assignments` table (composite PK).
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "issue_label_assignments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub issue_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub label_id: String,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project_issue::Entity",
        from = "Column::IssueId",
        to = "super::project_issue::Column::Id"
    )]
    ProjectIssue,
    #[sea_orm(
        belongs_to = "super::issue_label::Entity",
        from = "Column::LabelId",
        to = "super::issue_label::Column::Id"
    )]
    IssueLabel,
}

impl Related<super::project_issue::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectIssue.def()
    }
}

impl Related<super::issue_label::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::IssueLabel.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
