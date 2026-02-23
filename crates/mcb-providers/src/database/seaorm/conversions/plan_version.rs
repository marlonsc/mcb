//! PlanVersion domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::plan_version;
use mcb_domain::entities::plan::PlanVersion;

impl From<plan_version::Model> for PlanVersion {
    fn from(m: plan_version::Model) -> Self {
        Self {
            id: m.id,
            org_id: m.org_id,
            plan_id: m.plan_id,
            version_number: m.version_number,
            content_json: m.content_json,
            change_summary: m.change_summary,
            created_by: m.created_by,
            created_at: m.created_at,
        }
    }
}

impl From<PlanVersion> for plan_version::ActiveModel {
    fn from(e: PlanVersion) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            org_id: ActiveValue::Set(e.org_id),
            plan_id: ActiveValue::Set(e.plan_id),
            version_number: ActiveValue::Set(e.version_number),
            content_json: ActiveValue::Set(e.content_json),
            change_summary: ActiveValue::Set(e.change_summary),
            created_by: ActiveValue::Set(e.created_by),
            created_at: ActiveValue::Set(e.created_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_plan_version() -> PlanVersion {
        PlanVersion {
            id: "pv-001".into(),
            org_id: "org-001".into(),
            plan_id: "plan-001".into(),
            version_number: 1,
            content_json: r#"{"tasks":[]}"#.into(),
            change_summary: "Initial version".into(),
            created_by: "usr-001".into(),
            created_at: 1700000000,
        }
    }

    #[test]
    fn round_trip_plan_version() {
        let domain = sample_plan_version();
        let active: plan_version::ActiveModel = domain.clone().into();

        let model = plan_version::Model {
            id: active.id.unwrap(),
            org_id: active.org_id.unwrap(),
            plan_id: active.plan_id.unwrap(),
            version_number: active.version_number.unwrap(),
            content_json: active.content_json.unwrap(),
            change_summary: active.change_summary.unwrap(),
            created_by: active.created_by.unwrap(),
            created_at: active.created_at.unwrap(),
        };

        let back: PlanVersion = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.version_number, domain.version_number);
    }
}
