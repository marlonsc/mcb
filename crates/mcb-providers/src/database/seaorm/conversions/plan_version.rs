//! Plan version entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::plan_version;
use mcb_domain::entities::plan::PlanVersion;

crate::impl_conversion!(plan_version, PlanVersion,
    direct: [id, org_id, plan_id, version_number, content_json, change_summary, created_by, created_at]
);
