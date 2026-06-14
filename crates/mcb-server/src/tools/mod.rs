//! MCP Tools Module
//!
//! **Documentation**: [`docs/modules/server.md#tools`](../../../../docs/modules/server.md#tools)
//!
//! - registry.rs - Tool definitions and schema management
//! - router.rs - Tool dispatch and routing
//! - context.rs - Execution context extraction and resolution
//! - defaults.rs - Runtime defaults and execution flow configuration
//! - `field_aliases.rs` - Field alias resolution for metadata
//! - validation.rs - Execution context validation and hook processing

pub mod context;
pub mod defaults;
pub mod field_aliases;
pub mod registry;
pub mod router;
pub mod validation;

pub use context::ToolExecutionContext;
pub use defaults::{ExecutionFlow, RuntimeDefaults};
pub use registry::{create_tool_list, dispatch_tool_call, tool_by_name};
pub use router::{ToolHandlers, route_tool_call};
pub use validation::validate_execution_context;
