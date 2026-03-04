//! Project entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::project;
use mcb_domain::entities::Project;

crate::impl_conversion!(project, Project,
    direct: [id, org_id, name, path, created_at, updated_at]
);
