//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use std::future::Future;
use std::sync::Arc;

use mcb_domain::info;

use super::api::AdminApi;
use crate::transport::axum_http::{AppState, build_router};

impl AdminApi {
    /// Start the admin API server.
    ///
    /// # Errors
    /// Returns an error when Axum fails to bind or serve.
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("{}:{}", self.config.host, self.config.port)
            .parse::<std::net::SocketAddr>()
            .map_err(|e| std::io::Error::other(format!("Invalid admin bind address: {e}")))?;

        let app_state = Arc::new(AppState {
            metrics: Arc::clone(&self.state.metrics),
            indexing: Arc::clone(&self.state.indexing),
            browser: self.browse_state.as_ref().map(|b| Arc::clone(&b.browser)),
            browse_state: self.browse_state.map(Arc::new),
            mcp_server: None,
            admin_state: Some(Arc::new(self.state)),
            auth_config: Some(self.auth_config),
        });
        let router = build_router(&app_state);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!("ApiLaunch", "Admin API server listening", &addr);
        axum::serve(listener, router).await?;
        Ok(())
    }

    /// Start the admin API server with graceful shutdown.
    ///
    /// # Errors
    /// Returns an error when Axum fails to bind or serve.
    pub async fn start_with_shutdown(
        self,
        shutdown_signal: impl Future<Output = ()> + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("{}:{}", self.config.host, self.config.port)
            .parse::<std::net::SocketAddr>()
            .map_err(|e| std::io::Error::other(format!("Invalid admin bind address: {e}")))?;

        let app_state = Arc::new(AppState {
            metrics: Arc::clone(&self.state.metrics),
            indexing: Arc::clone(&self.state.indexing),
            browser: self.browse_state.as_ref().map(|b| Arc::clone(&b.browser)),
            browse_state: self.browse_state.map(Arc::new),
            mcp_server: None,
            admin_state: Some(Arc::new(self.state)),
            auth_config: Some(self.auth_config),
        });
        let router = build_router(&app_state);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!("ApiLaunch", "Admin API server listening", &addr);
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal)
            .await
            .map_err(|e| std::io::Error::other(format!("Admin Axum server failed: {e}")))?;
        Ok(())
    }
}
