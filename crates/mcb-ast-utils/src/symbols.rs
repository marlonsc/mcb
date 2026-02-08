//! Symbol Extraction
//!
//! Provides utilities for extracting function, class, and other symbol names
//! from parsed AST trees.

use mcb_language_support::LanguageId;
use tree_sitter::{Node, Tree};

use crate::cursor::CursorUtils;
use crate::walker::TreeWalker;

/// Information about an extracted symbol
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    /// Symbol name
    pub name: String,
    /// Symbol kind (function, class, method, etc.)
    pub kind: SymbolKind,
    /// Start line (0-indexed)
    pub start_line: usize,
    /// End line (0-indexed)
    pub end_line: usize,
    /// Start column (0-indexed)
    pub start_column: usize,
    /// Parent symbol name (for methods)
    pub parent: Option<String>,
}

/// Kind of symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// A function
    Function,
    /// A method (function inside a class/impl)
    Method,
    /// A class or struct
    Class,
    /// A module
    Module,
    /// A variable or constant
    Variable,
    /// An interface or trait
    Interface,
    /// An enum
    Enum,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolKind::Function => write!(f, "function"),
            SymbolKind::Method => write!(f, "method"),
            SymbolKind::Class => write!(f, "class"),
            SymbolKind::Module => write!(f, "module"),
            SymbolKind::Variable => write!(f, "variable"),
            SymbolKind::Interface => write!(f, "interface"),
            SymbolKind::Enum => write!(f, "enum"),
        }
    }
}

/// Symbol extractor for different languages
pub struct SymbolExtractor;

/// Symbol mapping type for language-specific extraction
type SymbolMapping = &'static [(&'static str, SymbolKind)];
/// Function checks type for language-specific extraction
type FunctionChecks = &'static [(&'static str, &'static str)];

impl SymbolExtractor {
    /// Extract all symbols from a tree
    pub fn extract(tree: &Tree, source: &[u8], language: LanguageId) -> Vec<SymbolInfo> {
        let root = tree.root_node();
        let mut symbols = Vec::new();

        let (mapping, checks): (SymbolMapping, FunctionChecks) = match language {
            LanguageId::Rust => (
                &[
                    ("struct_item", SymbolKind::Class),
                    ("trait_item", SymbolKind::Interface),
                    ("enum_item", SymbolKind::Enum),
                    ("mod_item", SymbolKind::Module),
                ],
                &[("function_item", "impl_item")],
            ),
            LanguageId::Python => (
                &[("class_definition", SymbolKind::Class)],
                &[("function_definition", "class_definition")],
            ),
            LanguageId::JavaScript | LanguageId::TypeScript => (
                &[
                    ("function_declaration", SymbolKind::Function),
                    ("method_definition", SymbolKind::Method),
                    ("class_declaration", SymbolKind::Class),
                ],
                &[],
            ),
            LanguageId::Java | LanguageId::Kotlin => (
                &[
                    ("method_declaration", SymbolKind::Method),
                    ("class_declaration", SymbolKind::Class),
                    ("interface_declaration", SymbolKind::Interface),
                ],
                &[],
            ),
            LanguageId::Cpp => (
                &[
                    ("function_definition", SymbolKind::Function),
                    ("class_specifier", SymbolKind::Class),
                    ("struct_specifier", SymbolKind::Class),
                ],
                &[],
            ),
        };

        for (node_kind, symbol_kind) in mapping {
            Self::extract_symbols_of_kind(root, source, node_kind, *symbol_kind, &mut symbols);
        }

        for (func_kind, parent_kind) in checks {
            Self::extract_functions_with_method_check(
                root,
                source,
                func_kind,
                parent_kind,
                &mut symbols,
            );
        }

        symbols
    }

    /// Extract function/method name from a node
    pub fn extract_name(node: Node<'_>, source: &[u8]) -> Option<String> {
        // Try common field names for function names
        for field in &["name", "declarator", "identifier"] {
            if let Some(name_node) = CursorUtils::child_by_field(node, field) {
                // Handle nested declarators (C/C++)
                if name_node.kind() == "function_declarator" || name_node.kind() == "declarator" {
                    return Self::extract_name(name_node, source);
                }
                if let Ok(name) = name_node.utf8_text(source) {
                    return Some(name.to_string());
                }
            }
        }

        // Fallback: find first identifier child
        CursorUtils::first_child_of_kind(node, "identifier")
            .and_then(|n| n.utf8_text(source).ok())
            .map(String::from)
    }

    /// Helper to extract symbols of a specific kind
    fn extract_symbols_of_kind(
        node: Node<'_>,
        source: &[u8],
        node_kind: &str,
        symbol_kind: SymbolKind,
        symbols: &mut Vec<SymbolInfo>,
    ) {
        for item in TreeWalker::find_by_kind(node, node_kind) {
            if let Some(name) = Self::extract_name(item, source) {
                symbols.push(SymbolInfo {
                    name,
                    kind: symbol_kind,
                    start_line: item.start_position().row,
                    end_line: item.end_position().row,
                    start_column: item.start_position().column,
                    parent: None,
                });
            }
        }
    }

    /// Helper to extract functions/methods checking for parent context
    fn extract_functions_with_method_check(
        node: Node<'_>,
        source: &[u8],
        func_kind: &str,
        parent_kind: &str,
        symbols: &mut Vec<SymbolInfo>,
    ) {
        for func in TreeWalker::find_by_kind(node, func_kind) {
            if let Some(name) = Self::extract_name(func, source) {
                let is_method = TreeWalker::is_inside_kind(func, parent_kind);
                symbols.push(SymbolInfo {
                    name,
                    kind: if is_method {
                        SymbolKind::Method
                    } else {
                        SymbolKind::Function
                    },
                    start_line: func.start_position().row,
                    end_line: func.end_position().row,
                    start_column: func.start_position().column,
                    parent: None,
                });
            }
        }
    }
}
