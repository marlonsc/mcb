//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Rule engine type identifiers.
//!
//! String constants for the pluggable rule engine system used in
//! YAML-based rule routing.

/// Rete network engine type.
pub const ENGINE_TYPE_RETE: &str = "rete";

/// Rust Rule Engine type.
pub const ENGINE_TYPE_RUST_RULE: &str = "rust-rule-engine";

/// GRL (Grule Rule Language) engine type.
pub const ENGINE_TYPE_GRL: &str = "grl";

/// Expression evaluator engine type.
pub const ENGINE_TYPE_EXPRESSION: &str = "expression";

/// `EvalExpr` engine type.
pub const ENGINE_TYPE_EVALEXPR: &str = "evalexpr";

/// Rusty Rules engine type.
pub const ENGINE_TYPE_RUSTY_RULES: &str = "rusty-rules";

/// JSON DSL engine type.
pub const ENGINE_TYPE_JSON_DSL: &str = "json-dsl";

// ============================================================================
// Linter Command Names
// ============================================================================

/// Ruff linter command name.
pub const LINTER_CMD_RUFF: &str = "ruff";

/// Cargo command name (for Clippy).
pub const LINTER_CMD_CARGO: &str = "cargo";
