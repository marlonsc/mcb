//! API Key entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::api_key;
use mcb_domain::entities::ApiKey;

crate::impl_conversion!(api_key, ApiKey,
    direct: [id, org_id, user_id, key_hash, name, scopes_json, expires_at, created_at, revoked_at]
);
