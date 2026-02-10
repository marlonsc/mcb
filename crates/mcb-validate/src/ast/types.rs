//! AST Types Module
//!
//! Additional AST-related types and violations.

use super::core::AstNode;

/// AST-based violation
#[derive(Debug, Clone)]
pub struct AstViolation {
    /// ID of the rule that was violated
    pub rule_id: String,
    /// Path to the file containing the violation
    pub file: String,
    /// The AST node that triggered the violation
    pub node: AstNode,
    /// Detailed error message
    pub message: String,
    /// Severity level
    pub severity: String,
}
