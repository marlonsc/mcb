//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! AST and tree-sitter node type constants

/// Node type for function definitions.
pub const NODE_FUNCTION_DEFINITION: &str = "function_definition";
/// Node type for function declarations.
pub const NODE_FUNCTION_DECLARATION: &str = "function_declaration";
/// Node type for method declarations.
pub const NODE_METHOD_DECLARATION: &str = "method_declaration";
/// Node type for class declarations.
pub const NODE_CLASS_DECLARATION: &str = "class_declaration";
/// Node type for interface declarations.
pub const NODE_INTERFACE_DECLARATION: &str = "interface_declaration";
/// Node type for struct specifiers.
pub const NODE_STRUCT_SPECIFIER: &str = "struct_specifier";

// Aliases for transition (to be removed once all crates are updated)
pub use NODE_CLASS_DECLARATION as AST_NODE_CLASS_DECLARATION;
pub use NODE_FUNCTION_DECLARATION as AST_NODE_FUNCTION_DECLARATION;
pub use NODE_FUNCTION_DEFINITION as AST_NODE_FUNCTION_DEFINITION;
pub use NODE_INTERFACE_DECLARATION as AST_NODE_INTERFACE_DECLARATION;
pub use NODE_METHOD_DECLARATION as AST_NODE_METHOD_DECLARATION;
pub use NODE_STRUCT_SPECIFIER as AST_NODE_STRUCT_SPECIFIER;

pub use NODE_CLASS_DECLARATION as TS_NODE_CLASS_DECLARATION;
pub use NODE_FUNCTION_DECLARATION as TS_NODE_FUNCTION_DECLARATION;
pub use NODE_FUNCTION_DEFINITION as TS_NODE_FUNCTION_DEFINITION;
pub use NODE_METHOD_DECLARATION as TS_NODE_METHOD_DECLARATION;
