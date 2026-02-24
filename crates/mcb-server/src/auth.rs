//!
//! **Documentation**: [docs/modules/server.md](../../../docs/modules/server.md)
//!
//! Authentication and Authorization
//!
//! This module provides API key authentication for admin endpoints.

use argon2::password_hash::PasswordHash;
use argon2::{Argon2, PasswordVerifier};
use axum::http::HeaderMap;
use loco_rs::app::AppContext;
use loco_rs::errors::Error;
use loco_rs::prelude::Result;
use mcb_infrastructure::config::AppConfig;
use mcb_providers::database::seaorm::entities::users;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;

const DEFAULT_API_KEY_HEADER: &str = "x-api-key";
const BEARER_PREFIX: &str = "Bearer ";

/// Authenticated admin principal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminPrincipal {
    /// User ID from the `users` table.
    pub user_id: String,
    /// User email from the `users` table.
    pub email: String,
    /// User role from the `users` table.
    pub role: String,
}

/// Authorize admin access using API key.
///
/// This checks `X-API-Key` (or `Authorization: Bearer <key>` fallback) against
/// `users.api_key_hash` entries.
///
/// # Errors
///
/// Returns `Unauthorized` when the key is missing or invalid.
pub async fn authorize_admin_api_key(
    ctx: &AppContext,
    headers: &HeaderMap,
) -> Result<AdminPrincipal> {
    let api_key_header = configured_api_key_header(ctx);
    let api_key = extract_api_key(headers, &api_key_header)?;

    let users_with_keys = users::Entity::find()
        .filter(users::Column::ApiKeyHash.is_not_null())
        .all(&ctx.db)
        .await?;

    for user in users_with_keys {
        if let Some(hash) = user.api_key_hash.as_deref()
            && verify_api_key(hash, &api_key)?
        {
            return Ok(AdminPrincipal {
                user_id: user.id,
                email: user.email,
                role: user.role,
            });
        }
    }

    Err(Error::Unauthorized("invalid api key".to_owned()))
}

fn configured_api_key_header(ctx: &AppContext) -> String {
    ctx.config
        .settings
        .as_ref()
        .and_then(|settings| serde_json::from_value::<AppConfig>(settings.clone()).ok())
        .map_or_else(
            || DEFAULT_API_KEY_HEADER.to_owned(),
            |cfg| cfg.auth.api_key.header.to_ascii_lowercase(),
        )
}

fn extract_api_key(headers: &HeaderMap, header_name: &str) -> Result<String> {
    if let Some(value) = headers.get(header_name) {
        let key = value
            .to_str()
            .map_err(|_| Error::Unauthorized("invalid api key header value".to_owned()))?
            .trim();
        if !key.is_empty() {
            return Ok(key.to_owned());
        }
    }

    if let Some(value) = headers.get("authorization") {
        let value = value
            .to_str()
            .map_err(|_| Error::Unauthorized("invalid authorization header value".to_owned()))?;
        if let Some(token) = value.strip_prefix(BEARER_PREFIX) {
            let key = token.trim();
            if !key.is_empty() {
                return Ok(key.to_owned());
            }
        }
    }

    Err(Error::Unauthorized(format!(
        "missing api key header ({header_name} or authorization bearer)"
    )))
}

fn verify_api_key(hash: &str, candidate: &str) -> Result<bool> {
    if let Ok(parsed) = PasswordHash::new(hash) {
        return Ok(Argon2::default()
            .verify_password(candidate.as_bytes(), &parsed)
            .is_ok());
    }

    if hash.starts_with("$2a$") || hash.starts_with("$2b$") || hash.starts_with("$2y$") {
        return bcrypt::verify(candidate, hash)
            .map_err(|e| {
                tracing::error!("bcrypt verification failed: {e}");
                Error::InternalServerError
            });
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn extract_api_key_reads_x_api_key() {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_static("abc123"));
        assert_eq!(
            extract_api_key(&headers, "x-api-key").expect("api key"),
            "abc123"
        );
    }

    #[test]
    fn extract_api_key_reads_authorization_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer abc123"));
        assert_eq!(
            extract_api_key(&headers, "x-api-key").expect("api key"),
            "abc123"
        );
    }

    #[test]
    fn extract_api_key_rejects_missing_headers() {
        let headers = HeaderMap::new();
        assert!(extract_api_key(&headers, "x-api-key").is_err());
    }
}
