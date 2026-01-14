//! MCP Context Browser - Enterprise Semantic Code Search Server
//!
//! An intelligent code analysis server powered by vector embeddings and advanced AI.
//! Transform natural language queries into precise code discoveries across large codebases.
//!
//! Features:
//! - 12 programming languages with AST parsing (Rust, Python, JavaScript, etc.)
//! - 6 embedding providers (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null)
//! - 6 vector stores (Milvus, EdgeVec, In-Memory, Filesystem, Encrypted, Null)
//! - Enterprise-grade security with SOC 2 compliance
//! - High performance (<500ms query responses, 1000+ concurrent users)

use clap::Parser;
use mcp_context_browser::server::run_server;

/// Command line interface for MCP Context Browser
#[derive(Parser, Debug)]
#[command(name = "mcp-context-browser")]
#[command(about = "MCP Context Browser - Semantic Code Search Server")]
#[command(version)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,
}

/// Main entry point for the MCP Context Browser server
///
/// Parses command line arguments and starts the MCP server.
/// The server will run until interrupted (Ctrl+C).
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    run_server(cli.config.as_deref()).await
}
