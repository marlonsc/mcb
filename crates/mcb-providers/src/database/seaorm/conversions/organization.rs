//! Organization entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::organization;
use mcb_domain::entities::Organization;

crate::impl_conversion!(organization, Organization,
    direct: [id, name, slug, settings_json, created_at, updated_at]
);
