//! Organization domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::organization;
use mcb_domain::entities::Organization;

impl From<organization::Model> for Organization {
    fn from(m: organization::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            slug: m.slug,
            settings_json: m.settings_json,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

impl From<Organization> for organization::ActiveModel {
    fn from(e: Organization) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            name: ActiveValue::Set(e.name),
            slug: ActiveValue::Set(e.slug),
            settings_json: ActiveValue::Set(e.settings_json),
            created_at: ActiveValue::Set(e.created_at),
            updated_at: ActiveValue::Set(e.updated_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_org() -> Organization {
        Organization {
            id: "org-001".into(),
            name: "Acme Corp".into(),
            slug: "acme-corp".into(),
            settings_json: r#"{"theme":"dark"}"#.into(),
            created_at: 1700000000,
            updated_at: 1700000001,
        }
    }

    #[test]
    fn round_trip_organization() {
        let domain = sample_org();
        let active: organization::ActiveModel = domain.clone().into();

        // Reconstruct Model from ActiveModel values
        let model = organization::Model {
            id: active.id.unwrap(),
            name: active.name.unwrap(),
            slug: active.slug.unwrap(),
            settings_json: active.settings_json.unwrap(),
            created_at: active.created_at.unwrap(),
            updated_at: active.updated_at.unwrap(),
        };

        let back: Organization = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.name, domain.name);
        assert_eq!(back.slug, domain.slug);
        assert_eq!(back.settings_json, domain.settings_json);
        assert_eq!(back.created_at, domain.created_at);
        assert_eq!(back.updated_at, domain.updated_at);
    }
}
