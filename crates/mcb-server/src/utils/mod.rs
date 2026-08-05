//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! General utility functions and helper modules for the MCB Server.
//!
//! This module contains shared logic that doesn't fit into a specific domain,
//! such as collection normalization and JSON handling.

/// Common argument resolution helpers for MCP handlers.
pub mod args;
/// Collection name normalization utilities.
pub mod collections;
pub mod json;
/// Shared helper functions for MCP tool handlers.
pub mod mcp;
