//! Plan domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::plan;
use mcb_domain::entities::Plan;
use mcb_domain::entities::plan::PlanStatus;

impl From<plan::Model> for Plan {
    fn from(m: plan::Model) -> Self {
        Self {
            id: m.id,
            org_id: m.org_id,
            project_id: m.project_id,
            title: m.title,
            description: m.description,
            status: m.status.parse::<PlanStatus>().unwrap_or(PlanStatus::Draft),
            created_by: m.created_by,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

impl From<Plan> for plan::ActiveModel {
    fn from(e: Plan) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            org_id: ActiveValue::Set(e.org_id),
            project_id: ActiveValue::Set(e.project_id),
            title: ActiveValue::Set(e.title),
            description: ActiveValue::Set(e.description),
            status: ActiveValue::Set(e.status.to_string()),
            created_by: ActiveValue::Set(e.created_by),
            created_at: ActiveValue::Set(e.created_at),
            updated_at: ActiveValue::Set(e.updated_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_plan() -> Plan {
        Plan {
            id: "plan-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            title: "v0.3.0 Roadmap".into(),
            description: "SeaORM migration plan".into(),
            status: PlanStatus::Active,
            created_by: "usr-001".into(),
            created_at: 1700000000,
            updated_at: 1700000001,
        }
    }

    #[test]
    fn round_trip_plan() {
        let domain = sample_plan();
        let active: plan::ActiveModel = domain.clone().into();

        let model = plan::Model {
            id: active.id.unwrap(),
            org_id: active.org_id.unwrap(),
            project_id: active.project_id.unwrap(),
            title: active.title.unwrap(),
            description: active.description.unwrap(),
            status: active.status.unwrap(),
            created_by: active.created_by.unwrap(),
            created_at: active.created_at.unwrap(),
            updated_at: active.updated_at.unwrap(),
        };

        let back: Plan = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.status, domain.status);
        assert_eq!(back.title, domain.title);
    }
}
