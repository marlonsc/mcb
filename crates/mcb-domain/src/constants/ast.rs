//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! AST and tree-sitter node type constants

/// AST node type for function definitions.
pub const AST_NODE_FUNCTION_DEFINITION: &str = "function_definition";
/// AST node type for function declarations.
pub const AST_NODE_FUNCTION_DECLARATION: &str = "function_declaration";
/// AST node type for method declarations.
pub const AST_NODE_METHOD_DECLARATION: &str = "method_declaration";
/// AST node type for class declarations.
pub const AST_NODE_CLASS_DECLARATION: &str = "class_declaration";
/// AST node type for interface declarations.
pub const AST_NODE_INTERFACE_DECLARATION: &str = "interface_declaration";
/// AST node type for struct specifiers.
pub const AST_NODE_STRUCT_SPECIFIER: &str = "struct_specifier";
/// Tree-sitter node type for function declarations.
pub const TS_NODE_FUNCTION_DECLARATION: &str = "function_declaration";
/// Tree-sitter node type for function definitions.
pub const TS_NODE_FUNCTION_DEFINITION: &str = "function_definition";
/// Tree-sitter node type for method declarations.
pub const TS_NODE_METHOD_DECLARATION: &str = "method_declaration";
/// Tree-sitter node type for class declarations.
pub const TS_NODE_CLASS_DECLARATION: &str = "class_declaration";
