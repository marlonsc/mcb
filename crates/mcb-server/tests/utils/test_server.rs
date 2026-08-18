//! Shared Axum test server for HTTP/MCP/UI integration tests.
//!
//! Provides [`TestServer`], a real TCP-bound Axum application built from the
//! production route table in [`mcb_server::axum_routes`]. Tests get a
//! `reqwest::Client` pre-configured with the bound base URL and can exercise
//! admin UI/API endpoints, auth middleware, and the MCP streamable HTTP
//! transport against an isolated database.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use axum::Router as AxumRouter;
use mcb_domain::entities::ApiKey;
use mcb_domain::utils::tests::utils::{create_test_admin_user, create_test_organization};
use mcb_server::axum_routes;
use mcb_server::state::McbState;
use mcb_utils::constants::auth::API_KEY_HEADER;
use reqwest::Client;
use tempfile::TempDir;

use crate::utils::domain_services::create_real_domain_services;

/// Admin credentials produced by [`TestServer::seed_admin_user`].
#[derive(Debug, Clone)]
pub struct AdminCredentials {
    /// Organization ID of the seeded admin.
    pub org_id: String,
    /// User ID of the seeded admin.
    pub user_id: String,
    /// Plaintext API key that authenticates the admin.
    pub api_key: String,
    /// Header name expected by the admin-auth middleware.
    pub header_name: String,
}

/// A real Axum server bound to an ephemeral loopback port.
///
/// The server runs in a background Tokio task. Dropping it aborts the task and
/// releases the TCP socket. The temporary directory holding the isolated `SQLite`
/// database is kept alive for the server's lifetime.
pub struct TestServer {
    /// Base URL of the running server, e.g. `http://127.0.0.1:12345`.
    pub base_url: String,
    /// Pre-configured HTTP client for the server.
    pub client: Client,
    /// Router handle, kept to ensure the service remains valid.
    #[allow(dead_code)]
    router: AxumRouter,
    /// Abort handle for the background serve task.
    abort_handle: tokio::task::AbortHandle,
    /// Temp directory with the isolated database.
    #[allow(dead_code)]
    temp_dir: TempDir,
    /// Shared MCB state, exposed for direct repository access in tests.
    pub state: McbState,
}

impl TestServer {
    /// Start a new [`TestServer`] with an isolated database and default route table.
    ///
    /// Returns `None` if provider resolution fails; the caller should skip the
    /// test in that case.
    pub async fn new() -> Option<Self> {
        let (state, temp_dir) = create_real_domain_services().await?;
        Self::with_state(state, temp_dir, None).await
    }

    /// Start a new [`TestServer`] with the given [`McbState`] and settings.
    ///
    /// `settings` is forwarded to the admin-auth middleware so tests can
    /// override the configured API-key header.
    pub async fn with_state(
        state: McbState,
        temp_dir: TempDir,
        settings: Option<serde_json::Value>,
    ) -> Option<Self> {
        let mcp_server = Arc::clone(&state.mcp_server);
        let router = axum_routes::build_full_router(state.clone(), mcp_server, settings);
        Self::start(router, state, temp_dir).await
    }

    /// Start the Axum application on an ephemeral loopback port.
    async fn start(router: AxumRouter, state: McbState, temp_dir: TempDir) -> Option<Self> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.ok()?;
        let addr: SocketAddr = listener.local_addr().ok()?;
        let base_url = format!("http://{addr}");

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .ok()?;

        let server = axum::serve(
            listener,
            router
                .clone()
                .into_make_service_with_connect_info::<SocketAddr>(),
        );
        let abort_handle = tokio::spawn(async move {
            if let Err(e) = server.await {
                tracing::warn!("TestServer serve task failed: {e}");
            }
        })
        .abort_handle();

        // Give the server a moment to start accepting connections.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        Some(Self {
            base_url,
            client,
            router,
            abort_handle,
            temp_dir,
            state,
        })
    }

    /// Return a full URL by appending `path` to [`Self::base_url`].
    #[must_use]
    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    /// GET request helper.
    ///
    /// # Errors
    /// Returns the underlying `reqwest` error if the request fails.
    pub async fn get(&self, path: &str) -> reqwest::Result<reqwest::Response> {
        self.client.get(self.url(path)).send().await
    }

    /// GET request helper that does not follow redirects.
    ///
    /// # Errors
    /// Returns the underlying `reqwest` error if building the client or the
    /// request fails.
    pub async fn get_no_redirect(&self, path: &str) -> reqwest::Result<reqwest::Response> {
        let no_redirect_client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        no_redirect_client.get(self.url(path)).send().await
    }

    /// POST request helper with a JSON body.
    ///
    /// # Errors
    /// Returns the underlying `reqwest` error if the request fails.
    pub async fn post<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> reqwest::Result<reqwest::Response> {
        self.client.post(self.url(path)).json(body).send().await
    }

    /// GET request with a custom API-key header.
    ///
    /// # Errors
    /// Returns the underlying `reqwest` error if the request fails.
    pub async fn get_with_key(
        &self,
        path: &str,
        header_name: &str,
        key: &str,
    ) -> reqwest::Result<reqwest::Response> {
        self.client
            .get(self.url(path))
            .header(header_name, key)
            .send()
            .await
    }

    /// POST request with a custom API-key header.
    ///
    /// # Errors
    /// Returns the underlying `reqwest` error if the request fails.
    pub async fn post_with_key<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
        header_name: &str,
        key: &str,
    ) -> reqwest::Result<reqwest::Response> {
        self.client
            .post(self.url(path))
            .header(header_name, key)
            .json(body)
            .send()
            .await
    }

    /// Create an admin user, organization, and API key in the isolated database.
    ///
    /// Returns the plaintext API key and the configured header name so tests can
    /// authenticate subsequent requests.
    ///
    /// # Errors
    /// Returns any repository, hashing, or validation error encountered while
    /// seeding the admin user.
    pub async fn seed_admin_user(&self) -> Result<AdminCredentials> {
        let org_id = mcb_utils::utils::id::generate().to_string();
        let org = create_test_organization(&org_id);
        let repo = self.state.mcp_server.org_entity_repository();

        repo.create_org(&org).await?;

        let mut user = create_test_admin_user(&org_id, "admin@test.example");
        let plaintext_key = format!("mcb-test-key-{}", mcb_utils::utils::id::generate());
        let key_hash = bcrypt::hash(&plaintext_key, 4)?;
        user.api_key_hash = Some(key_hash.clone());

        repo.create_user(&user).await?;

        let api_key = ApiKey {
            id: mcb_utils::utils::id::generate().to_string(),
            user_id: user.id.clone(),
            org_id: org_id.clone(),
            name: "test-admin-key".to_owned(),
            key_hash,
            scopes_json: "[]".to_owned(),
            expires_at: None,
            revoked_at: None,
            created_at: 0,
        };
        repo.create_api_key(&api_key).await?;

        Ok(AdminCredentials {
            org_id,
            user_id: user.id,
            api_key: plaintext_key,
            header_name: API_KEY_HEADER.to_ascii_lowercase(),
        })
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}

/// Test helper: assert that a response body contains the expected substring.
///
/// # Errors
/// Returns the underlying `reqwest` error if reading the response body fails.
///
/// # Panics
/// Panics if the body does not contain `expected`.
pub async fn assert_body_contains(response: reqwest::Response, expected: &str) -> Result<()> {
    let text = response.text().await?;
    assert!(
        text.contains(expected),
        "expected body to contain {expected:?}; got: {text}"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_starts_and_serves_public_asset() -> Result<()> {
        let Some(server) = TestServer::new().await else {
            return Ok(());
        };
        let response = server.get("/ui/theme.css").await?;
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        Ok(())
    }
}
