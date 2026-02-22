//! Integration tests for AST parsing and query execution
//!
//! These tests verify that:
//! - RCA parsers can parse multiple languages via `action()`
//! - `guess_language()` correctly detects languages
//! - AST queries can match patterns in code
//! - The unified AST format works correctly
//!
//! Using rust-code-analysis (RCA) directly - NO wrappers.

use rstest::rstest;
use std::path::Path;

use mcb_validate::ast::{
    AstNode, AstQuery, AstQueryBuilder, AstQueryPatterns, Position, QueryCondition, Span,
};
use rust_code_analysis::{Callback, LANG, ParserTrait, action, find, guess_language};

// ==================== RCA Callback for Parsing Tests ====================

/// Simple callback that returns the root node kind
struct RootKindCallback;

impl Callback for RootKindCallback {
    type Res = String;
    type Cfg = ();

    fn call<T: ParserTrait>(_cfg: Self::Cfg, parser: &T) -> Self::Res {
        parser.get_root().kind().to_owned()
    }
}

/// Callback that checks if a specific node type exists
struct HasNodeCallback;

impl Callback for HasNodeCallback {
    type Res = bool;
    type Cfg = String; // node type to find

    fn call<T: ParserTrait>(node_type: Self::Cfg, parser: &T) -> Self::Res {
        if let Some(nodes) = find(parser, &[node_type]) {
            !nodes.is_empty()
        } else {
            false
        }
    }
}

// ==================== Language Detection Tests (using RCA guess_language) ====================

#[rstest]
#[case(b"fn main() {}" as &[u8], "main.rs", Some(LANG::Rust))]
#[case(b"def main(): pass" as &[u8], "script.py", Some(LANG::Python))]
#[case(b"some content" as &[u8], "unknown.xyz", None)]
fn language_detection_exact(
    #[case] code: &[u8],
    #[case] file: &str,
    #[case] expected: Option<LANG>,
) {
    let (lang, _) = guess_language(code, Path::new(file));
    assert_eq!(lang, expected);
}

#[rstest]
#[case(b"function main() {}" as &[u8], "app.js", false)]
#[case(b"function main(): void {}" as &[u8], "component.ts", true)]
fn language_detection_js_family(#[case] code: &[u8], #[case] file: &str, #[case] is_ts: bool) {
    let (lang, _) = guess_language(code, Path::new(file));
    if is_ts {
        assert!(matches!(lang, Some(LANG::Typescript | LANG::Tsx)));
    } else {
        assert!(matches!(lang, Some(LANG::Javascript | LANG::Mozjs)));
    }
}

// Note: Go is not supported by rust-code-analysis
// Go tests removed as LANG::Go doesn't exist

// ==================== Rust Parser Tests (using RCA action) ====================

#[rstest]
#[case(
    LANG::Rust,
    br#"
fn hello_world() {
    println!("Hello, World!");
}
"# as &[u8],
    "test.rs",
    "source_file"
)]
#[case(
    LANG::Python,
    br#"
def hello_world():
    print("Hello, World!")
"# as &[u8],
    "test.py",
    "module"
)]
#[case(
    LANG::Javascript,
    br#"
function helloWorld() {
    console.log("Hello, World!");
}
"# as &[u8],
    "test.js",
    "program"
)]
#[case(
    LANG::Typescript,
    b"
function greet(name: string): string {
    return `Hello, ${name}!`;
}

interface Person {
    name: string;
    age: number;
}
" as &[u8],
    "test.ts",
    "program"
)]
#[case(
    LANG::Rust,
    b"
pub struct MyService {
    name: String,
    value: i32,
}

impl MyService {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: 0,
        }
    }
}
" as &[u8],
    "test.rs",
    "source_file"
)]
#[case(
    LANG::Python,
    b"
class MyService:
    def __init__(self, name: str):
        self.name = name
        self.value = 0

    def get_name(self) -> str:
        return self.name
" as &[u8],
    "test.py",
    "module"
)]
#[case(
    LANG::Javascript,
    b"
const add = (a, b) => a + b;
const multiply = (a, b) => {
    return a * b;
};
" as &[u8],
    "test.js",
    "program"
)]
fn parser_root_kind(
    #[case] lang: LANG,
    #[case] code: &[u8],
    #[case] file: &str,
    #[case] expected_root_kind: &str,
) {
    let root_kind = action::<RootKindCallback>(&lang, code.to_vec(), Path::new(file), None, ());
    assert_eq!(root_kind, expected_root_kind);
}

#[rstest]
#[case(
    LANG::Rust,
    br#"
fn risky_function() {
    let value = Some(42);
    let unwrapped = value.unwrap();
    println!("{}", unwrapped);
}
"# as &[u8],
    "test.rs",
    "function_item"
)]
#[case(
    LANG::Rust,
    b"
pub struct MyService {
    name: String,
    value: i32,
}

impl MyService {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: 0,
        }
    }
}
" as &[u8],
    "test.rs",
    "struct_item"
)]
#[case(
    LANG::Python,
    b"
class MyService:
    def __init__(self, name: str):
        self.name = name
        self.value = 0

    def get_name(self) -> str:
        return self.name
" as &[u8],
    "test.py",
    "class_definition"
)]
#[case(
    LANG::Javascript,
    b"
const add = (a, b) => a + b;
const multiply = (a, b) => {
    return a * b;
};
" as &[u8],
    "test.js",
    "arrow_function"
)]
fn parser_finds_node(
    #[case] lang: LANG,
    #[case] code: &[u8],
    #[case] file: &str,
    #[case] node_type: &str,
) {
    let has_node = action::<HasNodeCallback>(
        &lang,
        code.to_vec(),
        Path::new(file),
        None,
        node_type.into(),
    );
    assert!(has_node, "Should find {node_type} node");
}

// ==================== Python Parser Tests ====================

// ==================== Python/JavaScript/TypeScript Parser Tests ====================

// ==================== AST Query Tests ====================

#[test]
fn test_ast_query_builder() {
    let query = AstQueryBuilder::new("rust", "function_item")
        .with_condition(QueryCondition::Custom {
            name: "has_no_docstring".to_owned(),
        })
        .message("Function needs documentation")
        .severity("warning")
        .build();

    assert_eq!(query.language, "rust");
    assert_eq!(query.node_type, "function_item");
    assert_eq!(query.message, "Function needs documentation");
    assert_eq!(query.severity, "warning");
    assert_eq!(query.conditions.len(), 1);
}

#[test]
fn test_ast_query_patterns_undocumented_functions() {
    let query = AstQueryPatterns::undocumented_functions("rust");
    assert_eq!(query.language, "rust");
    assert_eq!(query.node_type, "function_item");
    assert_eq!(query.message, "Functions must be documented");
    assert_eq!(query.severity, "warning");
}

#[test]
fn test_ast_query_patterns_unwrap_usage() {
    let query = AstQueryPatterns::unwrap_usage("rust");
    assert_eq!(query.language, "rust");
    assert_eq!(query.node_type, "call_expression");
    assert_eq!(query.message, "Avoid unwrap() in production code");
    assert_eq!(query.severity, "error");
}

#[test]
fn test_ast_query_patterns_async_functions() {
    let query = AstQueryPatterns::async_functions("rust");
    assert_eq!(query.language, "rust");
    assert_eq!(query.node_type, "function_item");
    assert_eq!(query.message, "Async function detected");
    assert_eq!(query.severity, "info");
}

#[test]
fn test_ast_query_node_type_matching() {
    let query = AstQuery::new("rust", "identifier", "Found identifier", "info");

    let node = AstNode {
        kind: "identifier".to_owned(),
        name: Some("test".to_owned()),
        span: Span {
            start: Position {
                line: 1,
                column: 1,
                byte_offset: 0,
            },
            end: Position {
                line: 1,
                column: 5,
                byte_offset: 4,
            },
        },
        children: vec![],
        metadata: std::collections::HashMap::new(),
    };

    let violations = query.execute(&node);
    assert_eq!(violations.len(), 1, "Should match identifier node");
    assert!(violations[0].message.contains("Found identifier"));
}

#[test]
fn test_ast_query_no_match() {
    let query = AstQuery::new("rust", "function_item", "Found function", "info");

    let node = AstNode {
        kind: "identifier".to_owned(),
        name: Some("test".to_owned()),
        span: Span {
            start: Position {
                line: 1,
                column: 1,
                byte_offset: 0,
            },
            end: Position {
                line: 1,
                column: 5,
                byte_offset: 4,
            },
        },
        children: vec![],
        metadata: std::collections::HashMap::new(),
    };

    let violations = query.execute(&node);
    assert_eq!(violations.len(), 0, "Should not match non-function node");
}

#[test]
fn test_ast_query_recursive_matching() {
    let query = AstQuery::new("rust", "identifier", "Found identifier", "info");

    let child_node = AstNode {
        kind: "identifier".to_owned(),
        name: Some("inner".to_owned()),
        span: Span {
            start: Position {
                line: 2,
                column: 1,
                byte_offset: 10,
            },
            end: Position {
                line: 2,
                column: 6,
                byte_offset: 15,
            },
        },
        children: vec![],
        metadata: std::collections::HashMap::new(),
    };

    let root_node = AstNode {
        kind: "source_file".to_owned(),
        name: None,
        span: Span {
            start: Position {
                line: 1,
                column: 1,
                byte_offset: 0,
            },
            end: Position {
                line: 3,
                column: 1,
                byte_offset: 20,
            },
        },
        children: vec![child_node],
        metadata: std::collections::HashMap::new(),
    };

    let violations = query.execute(&root_node);
    assert_eq!(violations.len(), 1, "Should find identifier in children");
}

// ==================== Query Condition Tests ====================

#[test]
fn test_query_condition_has_child() {
    let query = AstQueryBuilder::new("rust", "function_item")
        .with_condition(QueryCondition::HasChild {
            child_type: "block".to_owned(),
        })
        .message("Function has block")
        .severity("info")
        .build();

    let block_node = AstNode {
        kind: "block".to_owned(),
        name: None,
        span: Span {
            start: Position {
                line: 1,
                column: 20,
                byte_offset: 20,
            },
            end: Position {
                line: 3,
                column: 1,
                byte_offset: 50,
            },
        },
        children: vec![],
        metadata: std::collections::HashMap::new(),
    };

    let func_node = AstNode {
        kind: "function_item".to_owned(),
        name: Some("test_fn".to_owned()),
        span: Span {
            start: Position {
                line: 1,
                column: 1,
                byte_offset: 0,
            },
            end: Position {
                line: 3,
                column: 1,
                byte_offset: 50,
            },
        },
        children: vec![block_node],
        metadata: std::collections::HashMap::new(),
    };

    let violations = query.execute(&func_node);
    assert_eq!(
        violations.len(),
        1,
        "Should match function with block child"
    );
}

#[test]
fn test_query_condition_no_child() {
    let query = AstQueryBuilder::new("rust", "function_item")
        .with_condition(QueryCondition::NoChild {
            child_type: "unsafe_block".to_owned(),
        })
        .message("Safe function")
        .severity("info")
        .build();

    let func_node = AstNode {
        kind: "function_item".to_owned(),
        name: Some("safe_fn".to_owned()),
        span: Span {
            start: Position {
                line: 1,
                column: 1,
                byte_offset: 0,
            },
            end: Position {
                line: 3,
                column: 1,
                byte_offset: 50,
            },
        },
        children: vec![],
        metadata: std::collections::HashMap::new(),
    };

    let violations = query.execute(&func_node);
    assert_eq!(
        violations.len(),
        1,
        "Should match function without unsafe block"
    );
}

// ==================== Multi-Language Query Tests ====================

#[rstest]
#[case("python", "function_definition")]
#[case("javascript", "function_declaration")]
#[case("go", "function_declaration")]
fn undocumented_function_query_patterns(#[case] language: &str, #[case] node_type: &str) {
    let query = AstQueryPatterns::undocumented_functions(language);
    assert_eq!(query.language, language);
    assert_eq!(query.node_type, node_type);
}
