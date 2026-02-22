//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Admin API Authentication
//!
//! Provides API key-based authentication for admin endpoints.
//! Uses the `X-Admin-Key` header by default (configurable).
//!
//! # Configuration
//!
//! Authentication can be configured via:
//! - Config file: `auth.admin.enabled = true` and `auth.admin.key = "your-key"`
//! - Environment variable: `MCP__AUTH__ADMIN__KEY=your-key`
//!
//! # Unauthenticated Routes
//!
//! The following routes are exempt from authentication:
//! - `/live` - Kubernetes liveness probe
//! - `/ready` - Kubernetes readiness probe

use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Admin authentication configuration for the middleware
#[derive(Clone)]
pub struct AdminAuthConfig {
    /// Whether authentication is enabled
    pub enabled: bool,
    /// The header name to check for the API key
    pub header_name: String,
    /// The expected API key value
    pub api_key: Option<String>,
}

impl AdminAuthConfig {
    /// Create a new admin auth config
    #[must_use]
    pub fn new(enabled: bool, header_name: String, api_key: Option<String>) -> Self {
        Self {
            enabled,
            header_name,
            api_key,
        }
    }

    /// Create from infrastructure config
    #[must_use]
    pub fn from_app_config(config: &mcb_infrastructure::config::AppConfig) -> Self {
        Self {
            enabled: config.auth.admin.enabled,
            header_name: config.auth.admin.header.clone(),
            api_key: config.auth.admin.key.clone(),
        }
    }

    /// Check if the provided key matches the configured key
    #[must_use]
    pub fn validate_key(&self, provided_key: &str) -> bool {
        match &self.api_key {
            Some(expected) => expected == provided_key,
            None => false, // If no key is configured, reject all requests
        }
    }

    /// Check if authentication is properly configured
    #[must_use]
    pub fn is_configured(&self) -> bool {
        self.enabled && self.api_key.is_some()
    }
}

impl Default for AdminAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            header_name: "X-Admin-Key".to_owned(),
            api_key: None,
        }
    }
}

/// Authentication error response
#[derive(Serialize)]
pub struct AuthErrorResponse {
    /// Error type
    pub error: &'static str,
    /// Error message
    pub message: String,
}

/// Check if a route should bypass authentication
#[must_use]
pub fn is_unauthenticated_route(path: &str) -> bool {
    matches!(path, "/live" | "/ready")
}

/// Admin authentication extractor for Axum handlers.
///
/// Implements [`axum::extract::FromRequestParts`] to extract and validate
/// the `X-Admin-Key` header (or the configured header name) against
/// [`AdminAuthConfig`], rejecting with a JSON error body on failure.
///
/// # Usage
///
/// ```rust,ignore
/// async fn protected_handler(
///     _auth: AxumAdminAuth,
///     State(state): State<Arc<AppState>>,
/// ) -> impl IntoResponse { /* ... */ }
/// ```
pub struct AxumAdminAuth;

/// JSON error body returned by the Axum auth extractor on rejection.
///
/// Mirrors the same `{ "error": "...", "message": "..." }` format used by
/// [`AuthErrorResponse`] in the Rocket guard.
#[derive(Debug, Serialize)]
pub struct AxumAuthError {
    /// Error type identifier (e.g. `"missing_api_key"`, `"invalid_api_key"`)
    pub error: &'static str,
    /// Human-readable error message
    pub message: String,
}

impl IntoResponse for AxumAuthError {
    fn into_response(self) -> Response {
        let status = match self.error {
            "auth_not_configured" => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::UNAUTHORIZED,
        };
        let body = serde_json::to_string(&self).unwrap_or_else(|_| {
            r#"{"error":"internal","message":"Failed to serialize error"}"#.to_owned()
        });
        (
            status,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            body,
        )
            .into_response()
    }
}

impl<S> axum::extract::FromRequestParts<S> for AxumAdminAuth
where
    S: Send + Sync,
{
    type Rejection = AxumAuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_config = match parts.extensions.get::<Arc<AdminAuthConfig>>() {
            Some(config) => Arc::clone(config),
            None => return Ok(Self),
        };

        if !auth_config.enabled {
            return Ok(Self);
        }

        if !auth_config.is_configured() {
            return Err(AxumAuthError {
                error: "auth_not_configured",
                message: "Admin authentication is enabled but no API key is configured. \
                         Set MCP__AUTH__ADMIN__KEY environment variable or auth.admin.key in config."
                    .to_owned(),
            });
        }

        let api_key = parts
            .headers
            .get(&auth_config.header_name)
            .and_then(|v| v.to_str().ok());

        match api_key {
            Some(key) if auth_config.validate_key(key) => Ok(Self),
            Some(_) => Err(AxumAuthError {
                error: "invalid_api_key",
                message: "Invalid admin API key".to_owned(),
            }),
            None => Err(AxumAuthError {
                error: "missing_api_key",
                message: format!(
                    "Admin API key required. Provide it in the '{}' header.",
                    auth_config.header_name
                ),
            }),
        }
    }
}

/// Axum middleware layer that injects [`AdminAuthConfig`] into request extensions.
///
/// Use with [`axum::middleware::from_fn_with_state`] or add as a layer:
///
/// ```rust,ignore
/// let app = Router::new()
///     .route("/protected", get(handler))
///     .layer(axum::Extension(Arc::new(auth_config)));
/// ```
///
/// Alternatively, use this function as a `from_fn` middleware that injects
/// the config and optionally short-circuits unauthenticated routes.
pub async fn axum_admin_auth_layer(
    axum::extract::Extension(config): axum::extract::Extension<Arc<AdminAuthConfig>>,
    mut request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Response {
    request.extensions_mut().insert(config);
    next.run(request).await
}
