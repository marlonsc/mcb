//! General utility functions and helper modules for the MCB Server.
//!
//! This module contains shared logic that doesn't fit into a specific domain,
//! such as collection normalization and JSON handling.

/// Collection name normalization utilities.
pub mod collections;
/// Handlebars custom helpers for the admin web UI.
pub mod handlebars;
pub mod json;
/// Shared helper functions for MCP tool handlers.
pub mod mcp;
/// Text extraction and processing utilities.
pub mod text;
