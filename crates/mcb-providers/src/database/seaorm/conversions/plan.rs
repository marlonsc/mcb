//! Plan entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::plan;
use mcb_domain::entities::Plan;
use mcb_domain::entities::plan::PlanStatus;

crate::impl_conversion!(plan, Plan,
    direct: [id, org_id, project_id, title, description, created_by, created_at, updated_at],
    enums: { status: PlanStatus = PlanStatus::Draft }
);
