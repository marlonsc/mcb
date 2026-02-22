//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Workflow FSM provider implementations for ADR-034.

pub mod transitions;

pub use transitions::apply_transition;
