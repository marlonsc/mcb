//! Organization and user registry implementations.
//!
//! Implements `OrgRegistry`, `UserRegistry`, and `ApiKeyRegistry` for managing
//! organizations, users, and API keys.

use super::*;

sea_impl_crud!(OrgRegistry for SeaOrmEntityRepository { db: db,
    entity: organization, domain: Organization, label: "Organization",
    create: create_org(org),
    get: get_org(id),
    list: list_orgs(),
    update: update_org(org),
    delete: delete_org(id)
});

#[async_trait]
impl UserRegistry for SeaOrmEntityRepository {
    async fn create_user(&self, u: &User) -> Result<()> {
        sea_repo_insert!(self.db(), user, u, "create user")
    }

    async fn get_user(&self, org_id: &str, id: &str) -> Result<User> {
        sea_repo_get_filtered!(self.db(), user, User, "User", id, "get user", user::Column::OrgId => org_id)
    }

    async fn get_user_by_email(&self, org_id: &str, email: &str) -> Result<User> {
        sea_repo_find_by_column!(self.db(), user, User, "User", email,
            "get user by email",
            user::Column::OrgId => org_id, user::Column::Email => email)
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
        sea_repo_set_field!(
            self.db(),
            api_key,
            id,
            "ApiKey",
            "revoke api key",
            revoked_at = Some(revoked_at)
        )
    }

    async fn delete_api_key(&self, id: &str) -> Result<()> {
        sea_repo_delete!(self.db(), api_key, id, "delete api key")
    }
}
