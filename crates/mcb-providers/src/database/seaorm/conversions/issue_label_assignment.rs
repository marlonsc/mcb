//! IssueLabelAssignment domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::issue_label_assignment;
use mcb_domain::entities::issue::IssueLabelAssignment;
use mcb_domain::value_objects::ids::IssueLabelAssignmentId;

impl From<issue_label_assignment::Model> for IssueLabelAssignment {
    fn from(m: issue_label_assignment::Model) -> Self {
        Self {
            id: IssueLabelAssignmentId::from(format!("{}:{}", m.issue_id, m.label_id).as_str()),
            issue_id: m.issue_id,
            label_id: m.label_id,
            created_at: m.created_at,
        }
    }
}

impl From<IssueLabelAssignment> for issue_label_assignment::ActiveModel {
    fn from(e: IssueLabelAssignment) -> Self {
        Self {
            issue_id: ActiveValue::Set(e.issue_id),
            label_id: ActiveValue::Set(e.label_id),
            created_at: ActiveValue::Set(e.created_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_label_assignment() -> IssueLabelAssignment {
        IssueLabelAssignment {
            id: IssueLabelAssignmentId::from("iss-001:lbl-001"),
            issue_id: "iss-001".into(),
            label_id: "lbl-001".into(),
            created_at: 1700000000,
        }
    }

    #[test]
    fn round_trip_issue_label_assignment() {
        let domain = sample_label_assignment();
        let active: issue_label_assignment::ActiveModel = domain.clone().into();

        let model = issue_label_assignment::Model {
            issue_id: active.issue_id.unwrap(),
            label_id: active.label_id.unwrap(),
            created_at: active.created_at.unwrap(),
        };

        let back: IssueLabelAssignment = model.into();
        assert_eq!(back.issue_id, domain.issue_id);
        assert_eq!(back.label_id, domain.label_id);
        assert_eq!(back.created_at, domain.created_at);
    }
}
