//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! General utility functions and helper modules for the MCB Server.
//!
//! This module contains shared logic that doesn't fit into a specific domain,
//! such as collection normalization and JSON handling.

/// Collection name normalization utilities.
pub mod collections;
pub mod json;
/// Shared helper functions for MCP tool handlers.
pub mod mcp;
/// Text extraction and processing utilities.
pub mod text;
