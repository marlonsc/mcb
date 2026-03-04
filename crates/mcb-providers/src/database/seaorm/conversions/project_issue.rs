//! Project issue entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::project_issue;
use mcb_domain::entities::ProjectIssue;
use mcb_domain::entities::project::{IssueStatus, IssueType};

crate::impl_conversion!(project_issue, ProjectIssue,
    direct: [id, org_id, project_id, created_by, phase_id, title, description, assignee,
             estimated_minutes, actual_minutes, notes, design, parent_issue_id,
             created_at, updated_at, closed_at, closed_reason],
    enums: {
        issue_type: IssueType = IssueType::Task,
        status: IssueStatus = IssueStatus::Open,
    },
    int_casts: [priority],
    json_arrays_req: [labels]
);
