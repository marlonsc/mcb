//! Symbol Extraction
//!
//! Provides utilities for extracting function, class, and other symbol names
//! from parsed AST trees.

use tree_sitter::{Node, Tree};

use crate::cursor::CursorUtils;
use crate::walker::TreeWalker;
use mcb_language_support::LanguageId;

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

impl SymbolExtractor {
    /// Extract all symbols from a tree
    pub fn extract(tree: &Tree, source: &[u8], language: LanguageId) -> Vec<SymbolInfo> {
        let root = tree.root_node();
        let mut symbols = Vec::new();

        match language {
            LanguageId::Rust => Self::extract_rust_symbols(root, source, &mut symbols),
            LanguageId::Python => Self::extract_python_symbols(root, source, &mut symbols),
            LanguageId::JavaScript | LanguageId::TypeScript => {
                Self::extract_js_symbols(root, source, &mut symbols);
            }
            LanguageId::Java | LanguageId::Kotlin => {
                Self::extract_java_symbols(root, source, &mut symbols);
            }
            LanguageId::Cpp => Self::extract_cpp_symbols(root, source, &mut symbols),
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

    fn extract_rust_symbols(node: Node<'_>, source: &[u8], symbols: &mut Vec<SymbolInfo>) {
        // Functions
        for func in TreeWalker::find_by_kind(node, "function_item") {
            if let Some(name) = Self::extract_name(func, source) {
                let is_method = TreeWalker::is_inside_kind(func, "impl_item");
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

        // Structs
        for item in TreeWalker::find_by_kind(node, "struct_item") {
            if let Some(name) = Self::extract_name(item, source) {
                symbols.push(SymbolInfo {
                    name,
                    kind: SymbolKind::Class,
                    start_line: item.start_position().row,
                    end_line: item.end_position().row,
                    start_column: item.start_position().column,
                    parent: None,
                });
            }
        }

        // Traits
        for item in TreeWalker::find_by_kind(node, "trait_item") {
            if let Some(name) = Self::extract_name(item, source) {
                symbols.push(SymbolInfo {
                    name,
                    kind: SymbolKind::Interface,
                    start_line: item.start_position().row,
                    end_line: item.end_position().row,
                    start_column: item.start_position().column,
                    parent: None,
                });
            }
        }

        // Enums
        for item in TreeWalker::find_by_kind(node, "enum_item") {
            if let Some(name) = Self::extract_name(item, source) {
                symbols.push(SymbolInfo {
                    name,
                    kind: SymbolKind::Enum,
                    start_line: item.start_position().row,
                    end_line: item.end_position().row,
                    start_column: item.start_position().column,
                    parent: None,
                });
            }
        }

        // Modules
        for item in TreeWalker::find_by_kind(node, "mod_item") {
            if let Some(name) = Self::extract_name(item, source) {
                symbols.push(SymbolInfo {
                    name,
                    kind: SymbolKind::Module,
                    start_line: item.start_position().row,
                    end_line: item.end_position().row,
                    start_column: item.start_position().column,
                    parent: None,
                });
            }
        }
    }

    fn extract_python_symbols(node: Node<'_>, source: &[u8], symbols: &mut Vec<SymbolInfo>) {
        // Functions
        for func in TreeWalker::find_by_kind(node, "function_definition") {
            if let Some(name) = Self::extract_name(func, source) {
                let is_method = TreeWalker::is_inside_kind(func, "class_definition");
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

        // Classes
        for class in TreeWalker::find_by_kind(node, "class_definition") {
            if let Some(name) = Self::extract_name(class, source) {
                symbols.push(SymbolInfo {
                    name,
                    kind: SymbolKind::Class,
                    start_line: class.start_position().row,
                    end_line: class.end_position().row,
                    start_column: class.start_position().column,
                    parent: None,
                });
            }
        }
    }

    fn extract_js_symbols(node: Node<'_>, source: &[u8], symbols: &mut Vec<SymbolInfo>) {
        // Function declarations
        for func in TreeWalker::find_by_kind(node, "function_declaration") {
            if let Some(name) = Self::extract_name(func, source) {
                symbols.push(SymbolInfo {
                    name,
                    kind: SymbolKind::Function,
                    start_line: func.start_position().row,
                    end_line: func.end_position().row,
                    start_column: func.start_position().column,
                    parent: None,
                });
            }
        }

        // Method definitions
        for method in TreeWalker::find_by_kind(node, "method_definition") {
            if let Some(name) = Self::extract_name(method, source) {
                symbols.push(SymbolInfo {
                    name,
                    kind: SymbolKind::Method,
                    start_line: method.start_position().row,
                    end_line: method.end_position().row,
                    start_column: method.start_position().column,
                    parent: None,
                });
            }
        }

        // Classes
        for class in TreeWalker::find_by_kind(node, "class_declaration") {
            if let Some(name) = Self::extract_name(class, source) {
                symbols.push(SymbolInfo {
                    name,
                    kind: SymbolKind::Class,
                    start_line: class.start_position().row,
                    end_line: class.end_position().row,
                    start_column: class.start_position().column,
                    parent: None,
                });
            }
        }
    }

    fn extract_java_symbols(node: Node<'_>, source: &[u8], symbols: &mut Vec<SymbolInfo>) {
        // Methods
        for method in TreeWalker::find_by_kind(node, "method_declaration") {
            if let Some(name) = Self::extract_name(method, source) {
                symbols.push(SymbolInfo {
                    name,
                    kind: SymbolKind::Method,
                    start_line: method.start_position().row,
                    end_line: method.end_position().row,
                    start_column: method.start_position().column,
                    parent: None,
                });
            }
        }

        // Classes
        for class in TreeWalker::find_by_kind(node, "class_declaration") {
            if let Some(name) = Self::extract_name(class, source) {
                symbols.push(SymbolInfo {
                    name,
                    kind: SymbolKind::Class,
                    start_line: class.start_position().row,
                    end_line: class.end_position().row,
                    start_column: class.start_position().column,
                    parent: None,
                });
            }
        }

        // Interfaces
        for iface in TreeWalker::find_by_kind(node, "interface_declaration") {
            if let Some(name) = Self::extract_name(iface, source) {
                symbols.push(SymbolInfo {
                    name,
                    kind: SymbolKind::Interface,
                    start_line: iface.start_position().row,
                    end_line: iface.end_position().row,
                    start_column: iface.start_position().column,
                    parent: None,
                });
            }
        }
    }

    fn extract_cpp_symbols(node: Node<'_>, source: &[u8], symbols: &mut Vec<SymbolInfo>) {
        // Functions
        for func in TreeWalker::find_by_kind(node, "function_definition") {
            if let Some(name) = Self::extract_name(func, source) {
                symbols.push(SymbolInfo {
                    name,
                    kind: SymbolKind::Function,
                    start_line: func.start_position().row,
                    end_line: func.end_position().row,
                    start_column: func.start_position().column,
                    parent: None,
                });
            }
        }

        // Classes and structs
        for class in TreeWalker::find_by_kind(node, "class_specifier") {
            if let Some(name) = Self::extract_name(class, source) {
                symbols.push(SymbolInfo {
                    name,
                    kind: SymbolKind::Class,
                    start_line: class.start_position().row,
                    end_line: class.end_position().row,
                    start_column: class.start_position().column,
                    parent: None,
                });
            }
        }

        for strct in TreeWalker::find_by_kind(node, "struct_specifier") {
            if let Some(name) = Self::extract_name(strct, source) {
                symbols.push(SymbolInfo {
                    name,
                    kind: SymbolKind::Class,
                    start_line: strct.start_position().row,
                    end_line: strct.end_position().row,
                    start_column: strct.start_position().column,
                    parent: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_rust(code: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Error loading Rust grammar");
        parser.parse(code, None).unwrap()
    }

    fn parse_python(code: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Error loading Python grammar");
        parser.parse(code, None).unwrap()
    }

    #[test]
    fn test_extract_rust_functions() {
        let code = "fn foo() {} fn bar() {}";
        let tree = parse_rust(code);
        let symbols = SymbolExtractor::extract(&tree, code.as_bytes(), LanguageId::Rust);

        assert_eq!(symbols.len(), 2);
        assert!(symbols.iter().any(|s| s.name == "foo"));
        assert!(symbols.iter().any(|s| s.name == "bar"));
        assert!(symbols.iter().all(|s| s.kind == SymbolKind::Function));
    }

    #[test]
    fn test_extract_rust_struct() {
        let code = "struct Foo { x: i32 }";
        let tree = parse_rust(code);
        let symbols = SymbolExtractor::extract(&tree, code.as_bytes(), LanguageId::Rust);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Foo");
        assert_eq!(symbols[0].kind, SymbolKind::Class);
    }

    #[test]
    fn test_extract_python_symbols() {
        let code = "def greet():\n    pass\n\nclass Person:\n    pass";
        let tree = parse_python(code);
        let symbols = SymbolExtractor::extract(&tree, code.as_bytes(), LanguageId::Python);

        assert!(symbols.iter().any(|s| s.name == "greet"));
        assert!(symbols.iter().any(|s| s.name == "Person"));
    }

    #[test]
    fn test_symbol_kind_display() {
        assert_eq!(SymbolKind::Function.to_string(), "function");
        assert_eq!(SymbolKind::Method.to_string(), "method");
        assert_eq!(SymbolKind::Class.to_string(), "class");
    }
}
