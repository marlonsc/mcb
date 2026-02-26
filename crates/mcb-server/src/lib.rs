//! MCB Server crate.
//!
//! MCP protocol server implementation with HTTP and stdio transports.
//!
//! ## Architecture
//! - `handlers` - MCP tool handlers for each domain operation
//! - `tools` - tool descriptor registry and dispatch wiring
//! - `transport` - HTTP and stdio transport adapters
//! - `session` - runtime session lifecycle and context handling
//!
//! [`composition::build_mcp_server_bootstrap`] is the composition-root that wires
//! domain services and repositories from the Loco application context.
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
mod macros;

pub mod args;
pub mod auth;
pub mod composition;
pub mod constants;
pub mod controllers;
pub mod error_mapping;
pub mod formatter;
pub mod handlers;
pub mod hooks;

/// Loco app hooks and MCP server composition root.
pub mod mcp_server;
pub mod session;
pub mod state;
pub mod tools;
/// Transport layer for MCP protocol communication.
pub mod transport;
pub mod utils;

mod exports;
pub use exports::*;
