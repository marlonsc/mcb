//! Organization and user registry implementations.
//!
//! Implements `OrgRegistry`, `UserRegistry`, and `ApiKeyRegistry` for managing
//! organizations, users, and API keys.

use super::*;

#[async_trait]
impl OrgRegistry for SeaOrmEntityRepository {
    async fn create_org(&self, org: &Organization) -> Result<()> {
        sea_repo_insert!(self.db(), organization, org, "create org")
    }

    async fn get_org(&self, id: &str) -> Result<Organization> {
        sea_repo_get!(
            self.db(),
            organization,
            Organization,
            "Organization",
            id,
            "get org"
        )
    }

    async fn list_orgs(&self) -> Result<Vec<Organization>> {
        sea_repo_list!(self.db(), organization, Organization, "list orgs")
    }

    async fn update_org(&self, org: &Organization) -> Result<()> {
        sea_repo_update!(self.db(), organization, org, "update org")
    }

    async fn delete_org(&self, id: &str) -> Result<()> {
        sea_repo_delete!(self.db(), organization, id, "delete org")
    }
}

#[async_trait]
impl UserRegistry for SeaOrmEntityRepository {
    async fn create_user(&self, u: &User) -> Result<()> {
        sea_repo_insert!(self.db(), user, u, "create user")
    }

    async fn get_user(&self, org_id: &str, id: &str) -> Result<User> {
        sea_repo_get_filtered!(self.db(), user, User, "User", id, "get user", user::Column::OrgId => org_id)
    }

    async fn get_user_by_email(&self, org_id: &str, email: &str) -> Result<User> {
        let model = user::Entity::find()
            .filter(user::Column::OrgId.eq(org_id))
            .filter(user::Column::Email.eq(email))
            .one(self.db())
            .await
            .map_err(crate::database::seaorm::repos::common::db_error(
                "get user by email",
            ))?;
        Error::not_found_or(model.map(User::from), "User", email)
    }

    async fn list_users(&self, org_id: &str) -> Result<Vec<User>> {
        sea_repo_list!(self.db(), user, User, "list users", user::Column::OrgId => org_id)
    }

    async fn update_user(&self, u: &User) -> Result<()> {
        sea_repo_update!(self.db(), user, u, "update user")
    }

    async fn delete_user(&self, id: &str) -> Result<()> {
        sea_repo_delete!(self.db(), user, id, "delete user")
    }
}

#[async_trait]
impl ApiKeyRegistry for SeaOrmEntityRepository {
    async fn create_api_key(&self, key: &ApiKey) -> Result<()> {
        sea_repo_insert!(self.db(), api_key, key, "create api key")
    }

    async fn get_api_key(&self, id: &str) -> Result<ApiKey> {
        sea_repo_get!(self.db(), api_key, ApiKey, "ApiKey", id, "get api key")
    }

    async fn list_api_keys(&self, org_id: &str) -> Result<Vec<ApiKey>> {
        sea_repo_list!(self.db(), api_key, ApiKey, "list api keys", api_key::Column::OrgId => org_id)
    }

    async fn revoke_api_key(&self, id: &str, revoked_at: i64) -> Result<()> {
        use sea_orm::ActiveValue;

        let model = api_key::Entity::find_by_id(id)
            .one(self.db())
            .await
            .map_err(crate::database::seaorm::repos::common::db_error(
                "revoke api key",
            ))?;
        let m = Error::not_found_or(model, "ApiKey", id)?;

        let mut active: api_key::ActiveModel = m.into();
        active.revoked_at = ActiveValue::Set(Some(revoked_at));
        active.update(self.db()).await.map_err(
            crate::database::seaorm::repos::common::db_error("revoke api key"),
        )?;
        Ok(())
    }

    async fn delete_api_key(&self, id: &str) -> Result<()> {
        sea_repo_delete!(self.db(), api_key, id, "delete api key")
    }
}
