//! Complexity Analysis
//!
//! Provides tree-sitter based complexity analysis independent of rust-code-analysis.
//! This module calculates cyclomatic and nesting complexity directly from AST.

use mcb_language_support::LanguageId;
use tree_sitter::Node;

use crate::cursor::CursorUtils;
use crate::walker::TreeWalker;

/// Complexity metrics for a code unit
#[derive(Debug, Clone, Default)]
pub struct ComplexityMetrics {
    /// Cyclomatic complexity (decision points + 1)
    pub cyclomatic: usize,
    /// Maximum nesting depth
    pub max_nesting: usize,
    /// Number of branches (if/else/match arms)
    pub branches: usize,
    /// Number of loops
    pub loops: usize,
}

/// Complexity analyzer using tree-sitter AST
pub struct ComplexityAnalyzer;

impl ComplexityAnalyzer {
    /// Analyze complexity of a node
    pub fn analyze(node: Node<'_>, language: LanguageId) -> ComplexityMetrics {
        let mut metrics = ComplexityMetrics {
            cyclomatic: 1, // Base complexity
            max_nesting: 0,
            branches: 0,
            loops: 0,
        };

        Self::analyze_recursive(node, language, 0, &mut metrics);
        metrics
    }

    fn analyze_recursive(
        node: Node<'_>,
        language: LanguageId,
        current_depth: usize,
        metrics: &mut ComplexityMetrics,
    ) {
        // Update max nesting for control flow nodes
        if Self::is_nesting_node(node.kind(), language) {
            let new_depth = current_depth + 1;
            if new_depth > metrics.max_nesting {
                metrics.max_nesting = new_depth;
            }
        }

        // Count decision points
        if Self::is_decision_point(node.kind(), language) {
            metrics.cyclomatic += 1;
            metrics.branches += 1;
        }

        // Count loops
        if Self::is_loop(node.kind(), language) {
            metrics.cyclomatic += 1;
            metrics.loops += 1;
        }

        // Recurse into children
        let depth_for_children = if Self::is_nesting_node(node.kind(), language) {
            current_depth + 1
        } else {
            current_depth
        };

        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                Self::analyze_recursive(cursor.node(), language, depth_for_children, metrics);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    fn is_decision_point(kind: &str, language: LanguageId) -> bool {
        match language {
            LanguageId::Rust => matches!(
                kind,
                "if_expression" | "else_clause" | "match_arm" | "and_expression" | "or_expression"
            ),
            LanguageId::Python => matches!(
                kind,
                "if_statement"
                    | "elif_clause"
                    | "else_clause"
                    | "and_operator"
                    | "or_operator"
                    | "conditional_expression"
            ),
            LanguageId::JavaScript | LanguageId::TypeScript => matches!(
                kind,
                "if_statement"
                    | "else_clause"
                    | "switch_case"
                    | "ternary_expression"
                    | "binary_expression" // && and ||
            ),
            LanguageId::Java | LanguageId::Kotlin => matches!(
                kind,
                "if_statement"
                    | "else_clause"
                    | "switch_expression"
                    | "ternary_expression"
                    | "binary_expression"
            ),
            LanguageId::Cpp => matches!(
                kind,
                "if_statement"
                    | "else_clause"
                    | "case_statement"
                    | "conditional_expression"
                    | "binary_expression"
            ),
        }
    }

    fn is_loop(kind: &str, language: LanguageId) -> bool {
        match language {
            LanguageId::Rust => {
                matches!(
                    kind,
                    "for_expression" | "while_expression" | "loop_expression"
                )
            }
            LanguageId::Python => matches!(kind, "for_statement" | "while_statement"),
            LanguageId::JavaScript | LanguageId::TypeScript => matches!(
                kind,
                "for_statement" | "for_in_statement" | "while_statement" | "do_statement"
            ),
            LanguageId::Java | LanguageId::Kotlin => {
                matches!(kind, "for_statement" | "while_statement" | "do_statement")
            }
            LanguageId::Cpp => matches!(
                kind,
                "for_statement" | "while_statement" | "do_statement" | "for_range_loop"
            ),
        }
    }

    fn is_nesting_node(kind: &str, language: LanguageId) -> bool {
        Self::is_decision_point(kind, language)
            || Self::is_loop(kind, language)
            || matches!(kind, "block" | "compound_statement" | "suite")
    }

    /// Calculate cognitive complexity (approximation based on nesting)
    ///
    /// Cognitive complexity adds extra weight for nested structures.
    pub fn cognitive_complexity(node: Node<'_>, language: LanguageId) -> usize {
        let mut total = 0;
        Self::cognitive_recursive(node, language, 0, &mut total);
        total
    }

    fn cognitive_recursive(
        node: Node<'_>,
        language: LanguageId,
        nesting_level: usize,
        total: &mut usize,
    ) {
        // Add complexity for decision points, weighted by nesting
        if Self::is_decision_point(node.kind(), language) || Self::is_loop(node.kind(), language) {
            *total += 1 + nesting_level;
        }

        // Increment nesting for appropriate nodes
        let new_nesting = if Self::is_nesting_node(node.kind(), language) {
            nesting_level + 1
        } else {
            nesting_level
        };

        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                Self::cognitive_recursive(cursor.node(), language, new_nesting, total);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }
}

/// Analyze nesting depth at a specific node
pub fn nesting_depth_at(node: Node<'_>) -> usize {
    TreeWalker::ancestors(node)
        .iter()
        .filter(|n| matches!(n.kind(), "block" | "compound_statement" | "suite"))
        .count()
}

/// Count parameters in a function node
pub fn count_parameters(function_node: Node<'_>) -> usize {
    // Try common parameter list field names
    for field in &["parameters", "formal_parameters", "parameter_list"] {
        if let Some(params) = CursorUtils::child_by_field(function_node, field) {
            return CursorUtils::named_children(params)
                .iter()
                .filter(|n| !matches!(n.kind(), "comment" | ","))
                .count();
        }
    }
    0
}

// Tests moved to tests/unit/complexity_tests.rs
