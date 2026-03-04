//! Issue label assignment entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::issue_label_assignment;
use mcb_domain::entities::issue::IssueLabelAssignment;
use mcb_domain::value_objects::ids::IssueLabelAssignmentId;

crate::impl_conversion!(issue_label_assignment, IssueLabelAssignment,
    direct: [issue_id, label_id, created_at],
    computed: { |m| id = IssueLabelAssignmentId::from(format!("{}:{}", m.issue_id, m.label_id).as_str()) },
    not_set: [id],
);
