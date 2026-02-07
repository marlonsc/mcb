//! AST Query Engine
//!
//! Provides querying capabilities over unified AST format for rule validation.
//!
//! # Example
//!
//! ```ignore
//! let query = AstQueryBuilder::new("rust", "function_item")
//!     .with_condition(QueryCondition::Custom { name: "has_no_docstring".to_string() })
//!     .message("Function needs documentation")
//!     .severity("warning")
//!     .build();
//! let violations = query.execute(&root_node);
//! ```

use regex::Regex;

use super::{AstNode, AstViolation};

/// AST query for pattern matching
#[derive(Debug, Clone)]
pub struct AstQuery {
    /// Target language for the query
    pub language: String,
    /// Target node kind
    pub node_type: String,
    /// List of conditions to match
    pub conditions: Vec<QueryCondition>,
    /// Violation message if query matches
    pub message: String,
    /// Severity level if query matches
    pub severity: String,
}

#[derive(Debug, Clone)]
pub enum QueryCondition {
    /// Node has specific field value
    HasField {
        /// Name of the field
        field: String,
        /// Expected value
        value: String,
    },
    /// Node does not have specific field
    NotHasField {
        /// Name of the field
        field: String,
    },
    /// Node name matches regex pattern
    NameMatches {
        /// Regex pattern to match
        pattern: String,
    },
    /// Node has children of specific type
    HasChild {
        /// Type of the child node
        child_type: String,
    },
    /// Node has no children of specific type
    NoChild {
        /// Type of the child node
        child_type: String,
    },
    /// Node has specific metadata value
    MetadataEquals {
        /// Metadata key
        key: String,
        /// Expected metadata value
        value: serde_json::Value,
    },
    /// Custom condition function
    Custom {
        /// Name of the custom condition
        name: String,
    },
}

impl AstQuery {
    /// Create a new AST query
    pub fn new(language: &str, node_type: &str, message: &str, severity: &str) -> Self {
        Self {
            language: language.to_string(),
            node_type: node_type.to_string(),
            conditions: Vec::new(),
            message: message.to_string(),
            severity: severity.to_string(),
        }
    }

    /// Add a condition to the query
    #[must_use]
    pub fn with_condition(mut self, condition: QueryCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Execute query on AST node
    pub fn execute(&self, node: &AstNode) -> Vec<AstViolation> {
        let mut violations = Vec::new();

        if self.matches_node(node) {
            violations.push(AstViolation {
                rule_id: format!("AST_{}_{}", self.language, self.node_type),
                file: "unknown".to_string(), // Would be set by caller
                node: node.clone(),
                message: self.message.clone(),
                severity: self.severity.clone(),
            });
        }

        // Recursively check children
        for child in &node.children {
            violations.extend(self.execute(child));
        }

        violations
    }

    /// Check if node matches query conditions
    fn matches_node(&self, node: &AstNode) -> bool {
        // Check node type
        if node.kind != self.node_type {
            return false;
        }

        // Check all conditions
        for condition in &self.conditions {
            if !self.check_condition(condition, node) {
                return false;
            }
        }

        true
    }

    /// Check individual condition
    fn check_condition(&self, condition: &QueryCondition, node: &AstNode) -> bool {
        match condition {
            QueryCondition::HasField { field, value } => node
                .metadata
                .get(field)
                .and_then(|v| v.as_str())
                .is_some_and(|s| s == value),
            QueryCondition::NotHasField { field } => !node.metadata.contains_key(field),
            QueryCondition::NameMatches { pattern } => match &node.name {
                Some(name) => Regex::new(pattern).ok().is_some_and(|re| re.is_match(name)),
                None => false,
            },
            QueryCondition::HasChild { child_type } => {
                node.children.iter().any(|child| child.kind == *child_type)
            }
            QueryCondition::NoChild { child_type } => {
                !node.children.iter().any(|child| child.kind == *child_type)
            }
            QueryCondition::MetadataEquals { key, value } => {
                node.metadata.get(key).is_some_and(|v| v == value)
            }
            QueryCondition::Custom { name } => self.check_custom_condition(name, node),
        }
    }

    /// Check custom conditions
    fn check_custom_condition(&self, name: &str, node: &AstNode) -> bool {
        match name {
            "has_no_docstring" => self.has_no_docstring(node),
            "is_async" => node
                .metadata
                .get("is_async")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            "has_return_type" => node
                .metadata
                .get("has_return_type")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            "is_test_function" => node
                .name
                .as_ref()
                .is_some_and(|n| n.starts_with("test_") || n.contains("test")),
            _ => false,
        }
    }

    /// Check if function has docstring/documentation
    fn has_no_docstring(&self, node: &AstNode) -> bool {
        // Look for documentation comments before the function
        // This is a simplified check - real implementation would
        // need to check source code around the node
        !node
            .metadata
            .get("has_docstring")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    }
}

/// Query builder for fluent API
pub struct AstQueryBuilder {
    language: String,
    node_type: String,
    conditions: Vec<QueryCondition>,
    message: String,
    severity: String,
}

impl AstQueryBuilder {
    /// Create a new query builder
    pub fn new(language: &str, node_type: &str) -> Self {
        Self {
            language: language.to_string(),
            node_type: node_type.to_string(),
            conditions: Vec::new(),
            message: String::new(),
            severity: "warning".to_string(),
        }
    }

    /// Add a condition to the query being built
    #[must_use]
    pub fn with_condition(mut self, condition: QueryCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Set the violation message
    #[must_use]
    pub fn message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    /// Set the violation severity
    #[must_use]
    pub fn severity(mut self, severity: &str) -> Self {
        self.severity = severity.to_string();
        self
    }

    /// Build the AST query
    #[must_use]
    pub fn build(self) -> AstQuery {
        AstQuery {
            language: self.language,
            node_type: self.node_type,
            conditions: self.conditions,
            message: self.message,
            severity: self.severity,
        }
    }
}

/// Returns the AST node type for function-like declarations for the given language.
fn function_node_type(language: &str) -> &'static str {
    match language {
        "rust" => "function_item",
        "python" => "function_definition",
        "javascript" | "typescript" | "go" => "function_declaration",
        _ => "function",
    }
}

/// Returns the AST node type for call expressions for the given language.
fn call_node_type(language: &str) -> &'static str {
    match language {
        "rust" | "javascript" | "typescript" | "go" => "call_expression",
        _ => "call", // python and others
    }
}

/// Common query patterns
pub struct AstQueryPatterns;

impl AstQueryPatterns {
    /// Query for functions without documentation
    pub fn undocumented_functions(language: &str) -> AstQuery {
        let node_type = function_node_type(language);

        AstQueryBuilder::new(language, node_type)
            .with_condition(QueryCondition::Custom {
                name: "has_no_docstring".to_string(),
            })
            .message("Functions must be documented")
            .severity("warning")
            .build()
    }

    /// Query for `unwrap()` usage in non-test code
    pub fn unwrap_usage(language: &str) -> AstQuery {
        let node_type = call_node_type(language);

        AstQueryBuilder::new(language, node_type)
            .with_condition(QueryCondition::HasField {
                field: "function_name".to_string(),
                value: "unwrap".to_string(),
            })
            .with_condition(QueryCondition::Custom {
                name: "is_test_function".to_string(),
            })
            .message("Avoid unwrap() in production code")
            .severity("error")
            .build()
    }

    /// Query for async functions
    pub fn async_functions(language: &str) -> AstQuery {
        let node_type = function_node_type(language);

        AstQueryBuilder::new(language, node_type)
            .with_condition(QueryCondition::Custom {
                name: "is_async".to_string(),
            })
            .message("Async function detected")
            .severity("info")
            .build()
    }
}
