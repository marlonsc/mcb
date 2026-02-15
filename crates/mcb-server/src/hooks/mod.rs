//! Hook processor for automatic memory operations on tool completion and session start.
//!
//! This module provides a hook system that automatically:
//! - Stores tool execution observations after tool completion (`PostToolUse`)
//! - Injects context from memory at session start (`SessionStart`)
//!
//! Hooks are optional and gracefully degrade if memory service is unavailable.

pub mod processor;
pub mod types;

pub use processor::HookProcessor;
pub use types::{
    Hook, HookContext, HookError, HookResult, PostToolUseContext, SessionStartContext,
};
