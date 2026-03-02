//! Organization and user registry implementations.
//!
//! Implements `OrgRegistry`, `UserRegistry`, and `ApiKeyRegistry` for managing
//! organizations, users, and API keys.

use super::*;

#[async_trait]
impl OrgRegistry for SeaOrmEntityRepository {
    async fn create_org(&self, org: &Organization) -> Result<()> {
        sea_insert!(self, organization, org)
    }

    async fn get_org(&self, id: &str) -> Result<Organization> {
        sea_get!(self, organization, Organization, "Organization", id)
    }

    async fn list_orgs(&self) -> Result<Vec<Organization>> {
        sea_list!(self, organization, Organization)
    }

    async fn update_org(&self, org: &Organization) -> Result<()> {
        sea_update!(self, organization, org)
    }

    async fn delete_org(&self, id: &str) -> Result<()> {
        sea_delete!(self, organization, id)
    }
}

#[async_trait]
impl UserRegistry for SeaOrmEntityRepository {
    async fn create_user(&self, u: &User) -> Result<()> {
        sea_insert!(self, user, u)
    }

    async fn get_user(&self, org_id: &str, id: &str) -> Result<User> {
        sea_get_filtered!(self, user, User, "User", id, user::Column::OrgId => org_id)
    }

    async fn get_user_by_email(&self, org_id: &str, email: &str) -> Result<User> {
        let model = user::Entity::find()
            .filter(user::Column::OrgId.eq(org_id))
            .filter(user::Column::Email.eq(email))
            .one(self.db())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(User::from), "User", email)
    }

    async fn list_users(&self, org_id: &str) -> Result<Vec<User>> {
        sea_list!(self, user, User, user::Column::OrgId => org_id)
    }

    async fn update_user(&self, u: &User) -> Result<()> {
        sea_update!(self, user, u)
    }

    async fn delete_user(&self, id: &str) -> Result<()> {
        sea_delete!(self, user, id)
    }
}

#[async_trait]
impl ApiKeyRegistry for SeaOrmEntityRepository {
    async fn create_api_key(&self, key: &ApiKey) -> Result<()> {
        sea_insert!(self, api_key, key)
    }

    async fn get_api_key(&self, id: &str) -> Result<ApiKey> {
        sea_get!(self, api_key, ApiKey, "ApiKey", id)
    }

    async fn list_api_keys(&self, org_id: &str) -> Result<Vec<ApiKey>> {
        sea_list!(self, api_key, ApiKey, api_key::Column::OrgId => org_id)
    }

    async fn revoke_api_key(&self, id: &str, revoked_at: i64) -> Result<()> {
        use sea_orm::ActiveValue;

        let model = api_key::Entity::find_by_id(id)
            .one(self.db())
            .await
            .map_err(db_err)?;
        let m = Error::not_found_or(model, "ApiKey", id)?;

        let mut active: api_key::ActiveModel = m.into();
        active.revoked_at = ActiveValue::Set(Some(revoked_at));
        active.update(self.db()).await.map_err(db_err)?;
        Ok(())
    }

    async fn delete_api_key(&self, id: &str) -> Result<()> {
        sea_delete!(self, api_key, id)
    }
}
