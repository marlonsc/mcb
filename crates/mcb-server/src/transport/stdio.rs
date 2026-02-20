//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Stdio Transport for MCP
//!
//! Implements MCP protocol over standard input/output streams.
//! This is the traditional transport mechanism for MCP servers.

use rmcp::ServiceExt;
use rmcp::transport::stdio;
use tracing::info;

use crate::McpServer;

/// Extension trait for `McpServer` to add stdio serving capability
///
/// # Example
///
/// ```no_run
/// use mcb_server::transport::StdioServerExt;
///
/// // let server = McpServer::new(context_service, config)?;
/// // server.serve_stdio().await?;  // Blocks until shutdown
/// ```
pub trait StdioServerExt {
    /// Serve the MCP server over stdio transport
    fn serve_stdio(
        self,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send;
}

impl StdioServerExt for McpServer {
    async fn serve_stdio(self) -> Result<(), Box<dyn std::error::Error>> {
        info!("📡 Starting MCP protocol server on stdio transport");

        let service = self
            .serve(stdio())
            .await
            .map_err(|e| format!("Failed to start MCP service: {e:?}"))?;

        info!("🎉 MCP server started successfully, waiting for connections...");
        service
            .waiting()
            .await
            .map_err(|e| format!("MCP service error: {e:?}"))?;

        info!("👋 MCP server shutdown complete");
        Ok(())
    }
}
