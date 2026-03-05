//! User entity ↔ `SeaORM` model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::user;
use mcb_domain::entities::User;
use mcb_domain::entities::user::UserRole;

crate::impl_conversion!(user, User,
    direct: [id, org_id, email, display_name, api_key_hash, created_at, updated_at],
    enums: { role: UserRole = UserRole::default() }
);
