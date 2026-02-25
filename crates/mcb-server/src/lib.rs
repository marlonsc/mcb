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
//! ## Entry point
//! [`loco_app::create_mcp_server`] is the composition-root helper that wires
//! domain services and repositories from the Loco application context.
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
mod macros;

pub mod args;
pub mod auth;
pub mod builder;
pub mod constants;
pub mod controllers;
pub mod error_mapping;
pub mod formatter;
pub mod handlers;
pub mod hooks;
pub mod initializers;
pub mod loco_app;
pub mod mcp_server;
pub mod session;
pub mod state;
pub mod tools;
/// Transport layer for MCP protocol communication.
pub mod transport;
pub mod utils;

mod exports;
pub use exports::*;
