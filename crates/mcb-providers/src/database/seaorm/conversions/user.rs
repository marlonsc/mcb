//! User domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::user;
use mcb_domain::entities::User;
use mcb_domain::entities::user::UserRole;

impl From<user::Model> for User {
    fn from(m: user::Model) -> Self {
        Self {
            id: m.id,
            org_id: m.org_id,
            email: m.email,
            display_name: m.display_name,
            role: m.role.parse::<UserRole>().unwrap_or_default(),
            api_key_hash: m.api_key_hash,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

impl From<User> for user::ActiveModel {
    fn from(e: User) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            org_id: ActiveValue::Set(e.org_id),
            email: ActiveValue::Set(e.email),
            display_name: ActiveValue::Set(e.display_name),
            role: ActiveValue::Set(e.role.to_string()),
            api_key_hash: ActiveValue::Set(e.api_key_hash),
            created_at: ActiveValue::Set(e.created_at),
            updated_at: ActiveValue::Set(e.updated_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_user() -> User {
        User {
            id: "usr-001".into(),
            org_id: "org-001".into(),
            email: "alice@example.com".into(),
            display_name: "Alice".into(),
            role: UserRole::Admin,
            api_key_hash: Some("hash123".into()),
            created_at: 1700000000,
            updated_at: 1700000001,
        }
    }

    #[test]
    fn round_trip_user() {
        let domain = sample_user();
        let active: user::ActiveModel = domain.clone().into();

        let model = user::Model {
            id: active.id.unwrap(),
            org_id: active.org_id.unwrap(),
            email: active.email.unwrap(),
            display_name: active.display_name.unwrap(),
            role: active.role.unwrap(),
            api_key_hash: active.api_key_hash.unwrap(),
            created_at: active.created_at.unwrap(),
            updated_at: active.updated_at.unwrap(),
        };

        let back: User = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.org_id, domain.org_id);
        assert_eq!(back.email, domain.email);
        assert_eq!(back.display_name, domain.display_name);
        assert_eq!(back.role, domain.role);
        assert_eq!(back.api_key_hash, domain.api_key_hash);
        assert_eq!(back.created_at, domain.created_at);
        assert_eq!(back.updated_at, domain.updated_at);
    }

    #[test]
    fn round_trip_user_no_api_key() {
        let mut domain = sample_user();
        domain.api_key_hash = None;
        let active: user::ActiveModel = domain.clone().into();

        let model = user::Model {
            id: active.id.unwrap(),
            org_id: active.org_id.unwrap(),
            email: active.email.unwrap(),
            display_name: active.display_name.unwrap(),
            role: active.role.unwrap(),
            api_key_hash: active.api_key_hash.unwrap(),
            created_at: active.created_at.unwrap(),
            updated_at: active.updated_at.unwrap(),
        };

        let back: User = model.into();
        assert_eq!(back.api_key_hash, None);
    }
}
