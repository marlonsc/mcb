//! MCP Tool Handlers
//!
//! Implementations of MCP tool calls using domain services.
//! Each handler translates MCP protocol requests into domain service calls.

pub mod browse_api;
pub mod browse_service;
pub mod consolidated;
pub mod highlight_service;

pub use consolidated::{
    AgentHandler, IndexHandler, MemoryHandler, ProjectHandler, SearchHandler, SessionHandler,
    ValidateHandler, VcsHandler,
};
pub use highlight_service::HighlightServiceImpl;
