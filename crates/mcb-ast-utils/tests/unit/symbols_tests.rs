//! Unit tests for symbol extraction
//!
//! Tests for `SymbolExtractor` functionality.

use mcb_ast_utils::symbols::{SymbolExtractor, SymbolKind};
use mcb_language_support::language::LanguageId;

use super::common::{parse_python, parse_rust};

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
