//! MCP Tool Handlers
//!
//! Implementations of MCP tool calls using domain services.
//! Each handler translates MCP protocol requests into domain service calls.

pub mod consolidated;

pub use consolidated::{
    AgentHandler, IndexHandler, MemoryHandler, ProjectHandler, SearchHandler, SessionHandler,
    ValidateHandler, VcsHandler,
};
pub use mcb_infrastructure::services::HighlightServiceImpl;
