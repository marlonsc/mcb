//! Team entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::team;
use mcb_domain::entities::Team;

crate::impl_conversion!(team, Team,
    direct: [id, org_id, name, created_at]
);
