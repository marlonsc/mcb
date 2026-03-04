//! Project decision entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::project_decision;
use mcb_domain::entities::project::ProjectDecision;

crate::impl_conversion!(project_decision, ProjectDecision,
    direct: [id, project_id, issue_id, title, context, decision, consequences, created_at]
);
