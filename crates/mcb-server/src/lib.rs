//! MCB server crate — MCP protocol server with HTTP and stdio transports.
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
mod macros;

pub mod args;
pub mod auth;
pub mod builder;
pub mod constants;
pub(crate) mod context_resolution;
pub mod error_mapping;
pub mod formatter;
pub mod handlers;
pub mod hooks;
pub mod loco_app;
pub mod mcp_server;
pub mod session;
pub mod tools;
/// Transport layer for MCP protocol communication.
pub mod transport;
pub mod utils;

pub use builder::McpServerBuilder;
pub use loco_app::McbApp;
pub use mcp_server::McpServer;
