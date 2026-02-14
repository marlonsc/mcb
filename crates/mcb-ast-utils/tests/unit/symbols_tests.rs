//! Unit tests for symbol extraction
//!
//! Tests for `SymbolExtractor` functionality.

use mcb_ast_utils::symbols::{SymbolExtractor, SymbolKind};
use mcb_language_support::language::LanguageId;
use rstest::rstest;

use super::common::{parse_python, parse_rust};

#[test]
fn extract_rust_functions() {
    let code = "fn foo() {} fn bar() {}";
    let tree = parse_rust(code);
    let symbols = SymbolExtractor::extract(&tree, code.as_bytes(), LanguageId::Rust);

    assert_eq!(symbols.len(), 2);
    assert!(symbols.iter().any(|s| s.name == "foo"));
    assert!(symbols.iter().any(|s| s.name == "bar"));
    assert!(symbols.iter().all(|s| s.kind == SymbolKind::Function));
}

#[test]
fn extract_rust_struct() {
    let code = "struct Foo { x: i32 }";
    let tree = parse_rust(code);
    let symbols = SymbolExtractor::extract(&tree, code.as_bytes(), LanguageId::Rust);

    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "Foo");
    assert_eq!(symbols[0].kind, SymbolKind::Class);
}

#[test]
fn extract_python_symbols() {
    let code = "def greet():\n    pass\n\nclass Person:\n    pass";
    let tree = parse_python(code);
    let symbols = SymbolExtractor::extract(&tree, code.as_bytes(), LanguageId::Python);

    assert!(symbols.iter().any(|s| s.name == "greet"));
    assert!(symbols.iter().any(|s| s.name == "Person"));
}

#[rstest]
#[case(SymbolKind::Function, "function")]
#[case(SymbolKind::Method, "method")]
#[case(SymbolKind::Class, "class")]
fn symbol_kind_display(#[case] kind: SymbolKind, #[case] expected: &str) {
    assert_eq!(kind.to_string(), expected);
}
