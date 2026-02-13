//! Agent Domain Module
//!
//! # Overview
//! This module defines the entities and logic for autonomous agents.
//! It covers the execution lifecycle, state management, and hierarchical delegation.
//!
//! # Core Entities
//! - [`AgentSession`]: The primary execution context for an agent task.
//! - [`Checkpoint`]: A snapshot of the session state for recovery or rollback.
//! - [`ToolCall`]: A record of an action performed by the agent.
//! - [`Delegation`]: A sub-task assigned to another agent or process.

mod checkpoint;
mod delegation;
mod session;
mod tool_call;
mod types;

pub use checkpoint::Checkpoint;
pub use delegation::Delegation;
pub use session::AgentSession;
pub use tool_call::ToolCall;
pub use types::{AgentSessionStatus, AgentType, CheckpointType};
