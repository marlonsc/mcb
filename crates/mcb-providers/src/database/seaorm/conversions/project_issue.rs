//! ProjectIssue domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::project_issue;
use mcb_domain::entities::ProjectIssue;
use mcb_domain::entities::project::{IssueStatus, IssueType};

impl From<project_issue::Model> for ProjectIssue {
    fn from(m: project_issue::Model) -> Self {
        let labels: Vec<String> = serde_json::from_str(&m.labels).unwrap_or_default();

        Self {
            id: m.id,
            org_id: m.org_id,
            project_id: m.project_id,
            created_by: m.created_by,
            phase_id: m.phase_id,
            title: m.title,
            description: m.description,
            issue_type: m.issue_type.parse::<IssueType>().unwrap_or(IssueType::Task),
            status: m.status.parse::<IssueStatus>().unwrap_or(IssueStatus::Open),
            priority: m.priority as i32,
            assignee: m.assignee,
            labels,
            estimated_minutes: m.estimated_minutes,
            actual_minutes: m.actual_minutes,
            notes: m.notes,
            design: m.design,
            parent_issue_id: m.parent_issue_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
            closed_at: m.closed_at,
            closed_reason: m.closed_reason,
        }
    }
}

impl From<ProjectIssue> for project_issue::ActiveModel {
    fn from(e: ProjectIssue) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            org_id: ActiveValue::Set(e.org_id),
            project_id: ActiveValue::Set(e.project_id),
            phase_id: ActiveValue::Set(e.phase_id),
            title: ActiveValue::Set(e.title),
            description: ActiveValue::Set(e.description),
            issue_type: ActiveValue::Set(e.issue_type.to_string()),
            status: ActiveValue::Set(e.status.to_string()),
            priority: ActiveValue::Set(i64::from(e.priority)),
            assignee: ActiveValue::Set(e.assignee),
            labels: ActiveValue::Set(
                serde_json::to_string(&e.labels).unwrap_or_else(|_| "[]".into()),
            ),
            created_at: ActiveValue::Set(e.created_at),
            updated_at: ActiveValue::Set(e.updated_at),
            closed_at: ActiveValue::Set(e.closed_at),
            created_by: ActiveValue::Set(e.created_by),
            estimated_minutes: ActiveValue::Set(e.estimated_minutes),
            actual_minutes: ActiveValue::Set(e.actual_minutes),
            notes: ActiveValue::Set(e.notes),
            design: ActiveValue::Set(e.design),
            parent_issue_id: ActiveValue::Set(e.parent_issue_id),
            closed_reason: ActiveValue::Set(e.closed_reason),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_project_issue() -> ProjectIssue {
        ProjectIssue {
            id: "iss-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            created_by: "usr-001".into(),
            phase_id: Some("phase-001".into()),
            title: "Fix auth bug".into(),
            description: "Auth fails on refresh".into(),
            issue_type: IssueType::Bug,
            status: IssueStatus::Open,
            priority: 1,
            assignee: Some("usr-002".into()),
            labels: vec!["bug".into(), "auth".into()],
            estimated_minutes: Some(120),
            actual_minutes: None,
            notes: "Needs investigation".into(),
            design: String::new(),
            parent_issue_id: None,
            created_at: 1700000000,
            updated_at: 1700000001,
            closed_at: None,
            closed_reason: String::new(),
        }
    }

    #[test]
    fn round_trip_project_issue() {
        let domain = sample_project_issue();
        let active: project_issue::ActiveModel = domain.clone().into();

        let model = project_issue::Model {
            id: active.id.unwrap(),
            org_id: active.org_id.unwrap(),
            project_id: active.project_id.unwrap(),
            phase_id: active.phase_id.unwrap(),
            title: active.title.unwrap(),
            description: active.description.unwrap(),
            issue_type: active.issue_type.unwrap(),
            status: active.status.unwrap(),
            priority: active.priority.unwrap(),
            assignee: active.assignee.unwrap(),
            labels: active.labels.unwrap(),
            created_at: active.created_at.unwrap(),
            updated_at: active.updated_at.unwrap(),
            closed_at: active.closed_at.unwrap(),
            created_by: active.created_by.unwrap(),
            estimated_minutes: active.estimated_minutes.unwrap(),
            actual_minutes: active.actual_minutes.unwrap(),
            notes: active.notes.unwrap(),
            design: active.design.unwrap(),
            parent_issue_id: active.parent_issue_id.unwrap(),
            closed_reason: active.closed_reason.unwrap(),
        };

        let back: ProjectIssue = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.issue_type, IssueType::Bug);
        assert_eq!(back.status, IssueStatus::Open);
        assert_eq!(back.labels, vec!["bug", "auth"]);
        assert_eq!(back.priority, 1);
    }
}
