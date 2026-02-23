//! IssueLabel domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::issue_label;
use mcb_domain::entities::IssueLabel;

impl From<issue_label::Model> for IssueLabel {
    fn from(m: issue_label::Model) -> Self {
        Self {
            id: m.id,
            org_id: m.org_id,
            project_id: m.project_id,
            name: m.name,
            color: m.color,
            created_at: m.created_at,
        }
    }
}

impl From<IssueLabel> for issue_label::ActiveModel {
    fn from(e: IssueLabel) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            org_id: ActiveValue::Set(e.org_id),
            project_id: ActiveValue::Set(e.project_id),
            name: ActiveValue::Set(e.name),
            color: ActiveValue::Set(e.color),
            created_at: ActiveValue::Set(e.created_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_issue_label() -> IssueLabel {
        IssueLabel {
            id: "lbl-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            name: "bug".into(),
            color: "#ff0000".into(),
            created_at: 1700000000,
        }
    }

    #[test]
    fn round_trip_issue_label() {
        let domain = sample_issue_label();
        let active: issue_label::ActiveModel = domain.clone().into();

        let model = issue_label::Model {
            id: active.id.unwrap(),
            org_id: active.org_id.unwrap(),
            project_id: active.project_id.unwrap(),
            name: active.name.unwrap(),
            color: active.color.unwrap(),
            created_at: active.created_at.unwrap(),
        };

        let back: IssueLabel = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.name, "bug");
        assert_eq!(back.color, "#ff0000");
    }
}
