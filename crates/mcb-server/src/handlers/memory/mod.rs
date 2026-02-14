//! Memory handler for observations, executions, quality gates, and session summaries.
//!
//! This module provides a unified handler for memory-related MCP tool operations.

mod execution;
mod handler;
mod inject;
mod list_timeline;
mod observation;
mod quality_gate;
mod session;

pub use handler::MemoryHandler;
