use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::collections::HashMap;

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
        let mut api_key_rows = api_keys::Entity::find()
            .filter(api_keys::Column::RevokedAt.is_null())
            .all(&self.db)
            .await
            .map_err(|e| Error::database(format!("find API key records failed: {e}")))?;

        if api_key_rows.iter().any(|row| row.key_hash == key_hash) {
            api_key_rows.retain(|row| row.key_hash == key_hash);
        }

        if api_key_rows.is_empty() {
            return Ok(Vec::new());
        }

        let user_ids: Vec<String> = api_key_rows.iter().map(|row| row.user_id.clone()).collect();
        let users_with_keys = users::Entity::find()
            .filter(users::Column::Id.is_in(user_ids))
            .all(&self.db)
            .await
            .map_err(|e| Error::database(format!("find users with API keys failed: {e}")))?;

        let user_by_id: HashMap<String, users::Model> = users_with_keys
            .into_iter()
            .map(|user| (user.id.clone(), user))
            .collect();

        api_key_rows
            .into_iter()
            .filter_map(|api_key_model| {
                user_by_id
                    .get(&api_key_model.user_id)
                    .cloned()
                    .map(|user_model| (api_key_model, user_model))
            })
            .map(|(api_key_model, user_model)| {
                let user = Self::map_user(user_model)?;
                Ok(UserWithApiKey {
                    user,
                    api_key_id: api_key_model.id,
                    api_key_hash: api_key_model.key_hash,
                })
            })
            .collect()
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
