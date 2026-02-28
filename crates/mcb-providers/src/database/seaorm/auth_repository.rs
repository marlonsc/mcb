use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use mcb_domain::entities::User;
use mcb_domain::entities::user::UserRole;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{ApiKeyInfo, AuthRepositoryPort, UserWithApiKey};

use crate::database::seaorm::entities::{api_keys, users};

/// SeaORM-backed implementation of [`AuthRepositoryPort`].
pub struct SeaOrmAuthRepositoryAdapter {
    db: DatabaseConnection,
}

impl SeaOrmAuthRepositoryAdapter {
    /// Creates an adapter using the given database connection.
    #[must_use]
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn map_user(model: users::Model) -> Result<User> {
        Ok(User {
            id: model.id,
            org_id: model.org_id,
            email: model.email,
            display_name: model.display_name,
            role: model.role.parse::<UserRole>().map_err(|e| {
                Error::authentication(format!("Invalid user role '{}': {}", model.role, e))
            })?,
            api_key_hash: model.api_key_hash,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}

#[async_trait]
impl AuthRepositoryPort for SeaOrmAuthRepositoryAdapter {
    async fn find_users_by_api_key_hash(&self, key_hash: &str) -> Result<Vec<UserWithApiKey>> {
        let users_with_keys = users::Entity::find()
            .filter(users::Column::ApiKeyHash.eq(key_hash.to_owned()))
            .all(&self.db)
            .await
            .map_err(|e| Error::database(format!("find users with API key hashes failed: {e}")))?;

        let results = users_with_keys
            .into_iter()
            .filter_map(|user_model| {
                let user = Self::map_user(user_model.clone()).ok()?;
                let api_key_hash = user_model.api_key_hash?;
                Some(UserWithApiKey {
                    user,
                    api_key_id: user_model.id,
                    api_key_hash,
                })
            })
            .collect();
        Ok(results)
    }

    async fn verify_api_key(&self, key_hash: &str) -> Result<Option<ApiKeyInfo>> {
        let api_key = api_keys::Entity::find()
            .filter(api_keys::Column::KeyHash.eq(key_hash.to_owned()))
            .one(&self.db)
            .await
            .map_err(|e| Error::database(format!("verify API key failed: {e}")))?;

        Ok(api_key.map(|row| ApiKeyInfo {
            api_key_id: row.id,
            user_id: row.user_id,
            organization_id: Some(row.org_id),
        }))
    }
}
