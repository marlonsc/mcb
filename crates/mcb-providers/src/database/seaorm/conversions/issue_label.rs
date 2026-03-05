//! Issue label entity ↔ `SeaORM` model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::issue_label;
use mcb_domain::entities::IssueLabel;

crate::impl_conversion!(issue_label, IssueLabel,
    direct: [id, org_id, project_id, name, color, created_at]
);
