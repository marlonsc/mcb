//! ApiKey domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::api_key;
use mcb_domain::entities::ApiKey;

impl From<api_key::Model> for ApiKey {
    fn from(m: api_key::Model) -> Self {
        Self {
            id: m.id,
            org_id: m.org_id,
            user_id: m.user_id,
            key_hash: m.key_hash,
            name: m.name,
            scopes_json: m.scopes_json,
            expires_at: m.expires_at,
            created_at: m.created_at,
            revoked_at: m.revoked_at,
        }
    }
}

impl From<ApiKey> for api_key::ActiveModel {
    fn from(e: ApiKey) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            user_id: ActiveValue::Set(e.user_id),
            org_id: ActiveValue::Set(e.org_id),
            key_hash: ActiveValue::Set(e.key_hash),
            name: ActiveValue::Set(e.name),
            scopes_json: ActiveValue::Set(e.scopes_json),
            expires_at: ActiveValue::Set(e.expires_at),
            created_at: ActiveValue::Set(e.created_at),
            revoked_at: ActiveValue::Set(e.revoked_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_api_key() -> ApiKey {
        ApiKey {
            id: "key-001".into(),
            org_id: "org-001".into(),
            user_id: "usr-001".into(),
            key_hash: "sha256:abc123".into(),
            name: "CI Key".into(),
            scopes_json: r#"["read","write"]"#.into(),
            expires_at: Some(1800000000),
            created_at: 1700000000,
            revoked_at: None,
        }
    }

    #[test]
    fn round_trip_api_key() {
        let domain = sample_api_key();
        let active: api_key::ActiveModel = domain.clone().into();

        let model = api_key::Model {
            id: active.id.unwrap(),
            user_id: active.user_id.unwrap(),
            org_id: active.org_id.unwrap(),
            key_hash: active.key_hash.unwrap(),
            name: active.name.unwrap(),
            scopes_json: active.scopes_json.unwrap(),
            expires_at: active.expires_at.unwrap(),
            created_at: active.created_at.unwrap(),
            revoked_at: active.revoked_at.unwrap(),
        };

        let back: ApiKey = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.expires_at, domain.expires_at);
        assert_eq!(back.revoked_at, None);
    }
}
