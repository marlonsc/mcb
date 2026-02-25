//! MCP Tools Module
//!
//! **Documentation**: [`docs/modules/server.md#tools`](../../../../docs/modules/server.md#tools)
//!
//! - registry.rs - Tool definitions and schema management
//! - router.rs - Tool dispatch and routing

pub mod registry;
pub mod router;

pub use registry::{ToolDefinitions, create_tool_list, dispatch_tool_call};
pub use router::{
    ExecutionFlow, RuntimeDefaults, ToolExecutionContext, ToolHandlers, route_tool_call,
};
