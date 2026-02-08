//! Integration tests for AST parsing and query execution
//!
//! These tests verify that:
//! - RCA parsers can parse multiple languages via `action()`
//! - `guess_language()` correctly detects languages
//! - AST queries can match patterns in code
//! - The unified AST format works correctly
//!
//! Using rust-code-analysis (RCA) directly - NO wrappers.

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
        parser.get_root().kind().to_string()
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

#[test]
fn test_language_detection_rust() {
    let code = b"fn main() {}";
    let (lang, _) = guess_language(code, Path::new("main.rs"));
    assert_eq!(lang, Some(LANG::Rust));
}

#[test]
fn test_language_detection_python() {
    let code = b"def main(): pass";
    let (lang, _) = guess_language(code, Path::new("script.py"));
    assert_eq!(lang, Some(LANG::Python));
}

#[test]
fn test_language_detection_javascript() {
    let code = b"function main() {}";
    let (lang, _) = guess_language(code, Path::new("app.js"));
    // JavaScript is detected as Mozjs in RCA
    assert!(matches!(lang, Some(LANG::Javascript | LANG::Mozjs)));
}

#[test]
fn test_language_detection_typescript() {
    let code = b"function main(): void {}";
    let (lang, _) = guess_language(code, Path::new("component.ts"));
    assert!(matches!(lang, Some(LANG::Typescript | LANG::Tsx)));
}

// Note: Go is not supported by rust-code-analysis
// Go tests removed as LANG::Go doesn't exist

#[test]
fn test_language_detection_unknown() {
    let code = b"some content";
    let (lang, _) = guess_language(code, Path::new("unknown.xyz"));
    assert_eq!(lang, None);
}

// ==================== Rust Parser Tests (using RCA action) ====================

#[test]
fn test_rust_parser_simple_function() {
    let code = br#"
fn hello_world() {
    println!("Hello, World!");
}
"#;
    let path = Path::new("test.rs");

    let root_kind = action::<RootKindCallback>(&LANG::Rust, code.to_vec(), path, None, ());
    assert_eq!(root_kind, "source_file");
}

#[test]
fn test_rust_parser_finds_function() {
    let code = br#"
fn risky_function() {
    let value = Some(42);
    let unwrapped = value.unwrap();
    println!("{}", unwrapped);
}
"#;
    let path = Path::new("test.rs");

    let has_function = action::<HasNodeCallback>(
        &LANG::Rust,
        code.to_vec(),
        path,
        None,
        "function_item".into(),
    );
    assert!(has_function, "Should find function_item node");
}

#[test]
fn test_rust_parser_struct() {
    let code = br"
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
";
    let path = Path::new("test.rs");

    let root_kind = action::<RootKindCallback>(&LANG::Rust, code.to_vec(), path, None, ());
    assert_eq!(root_kind, "source_file");

    let has_struct =
        action::<HasNodeCallback>(&LANG::Rust, code.to_vec(), path, None, "struct_item".into());
    assert!(has_struct, "Should find struct_item node");
}

// ==================== Python Parser Tests ====================

#[test]
fn test_python_parser_simple_function() {
    let code = br#"
def hello_world():
    print("Hello, World!")
"#;
    let path = Path::new("test.py");

    let root_kind = action::<RootKindCallback>(&LANG::Python, code.to_vec(), path, None, ());
    assert_eq!(root_kind, "module");
}

#[test]
fn test_python_parser_class() {
    let code = br"
class MyService:
    def __init__(self, name: str):
        self.name = name
        self.value = 0

    def get_name(self) -> str:
        return self.name
";
    let path = Path::new("test.py");

    let root_kind = action::<RootKindCallback>(&LANG::Python, code.to_vec(), path, None, ());
    assert_eq!(root_kind, "module");

    let has_class = action::<HasNodeCallback>(
        &LANG::Python,
        code.to_vec(),
        path,
        None,
        "class_definition".into(),
    );
    assert!(has_class, "Should find class_definition node");
}

// ==================== JavaScript Parser Tests ====================

#[test]
fn test_javascript_parser_simple_function() {
    let code = br#"
function helloWorld() {
    console.log("Hello, World!");
}
"#;
    let path = Path::new("test.js");

    let root_kind = action::<RootKindCallback>(&LANG::Javascript, code.to_vec(), path, None, ());
    assert_eq!(root_kind, "program");
}

#[test]
fn test_javascript_parser_arrow_function() {
    let code = br"
const add = (a, b) => a + b;
const multiply = (a, b) => {
    return a * b;
};
";
    let path = Path::new("test.js");

    let root_kind = action::<RootKindCallback>(&LANG::Javascript, code.to_vec(), path, None, ());
    assert_eq!(root_kind, "program");

    let has_arrow = action::<HasNodeCallback>(
        &LANG::Javascript,
        code.to_vec(),
        path,
        None,
        "arrow_function".into(),
    );
    assert!(has_arrow, "Should find arrow_function node");
}

// ==================== TypeScript Parser Tests ====================

#[test]
fn test_typescript_parser_typed_function() {
    let code = br"
function greet(name: string): string {
    return `Hello, ${name}!`;
}

interface Person {
    name: string;
    age: number;
}
";
    let path = Path::new("test.ts");

    let root_kind = action::<RootKindCallback>(&LANG::Typescript, code.to_vec(), path, None, ());
    assert_eq!(root_kind, "program");
}

// Note: Go parser tests removed - LANG::Go doesn't exist in rust-code-analysis

// ==================== AST Query Tests (unchanged - uses internal AstNode) ====================

#[test]
fn test_ast_query_builder() {
    let query = AstQueryBuilder::new("rust", "function_item")
        .with_condition(QueryCondition::Custom {
            name: "has_no_docstring".to_string(),
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
        kind: "identifier".to_string(),
        name: Some("test".to_string()),
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
        kind: "identifier".to_string(),
        name: Some("test".to_string()),
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
        kind: "identifier".to_string(),
        name: Some("inner".to_string()),
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
        kind: "source_file".to_string(),
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
            child_type: "block".to_string(),
        })
        .message("Function has block")
        .severity("info")
        .build();

    let block_node = AstNode {
        kind: "block".to_string(),
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
        kind: "function_item".to_string(),
        name: Some("test_fn".to_string()),
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
            child_type: "unsafe_block".to_string(),
        })
        .message("Safe function")
        .severity("info")
        .build();

    let func_node = AstNode {
        kind: "function_item".to_string(),
        name: Some("safe_fn".to_string()),
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

#[test]
fn test_python_query_patterns() {
    let query = AstQueryPatterns::undocumented_functions("python");
    assert_eq!(query.language, "python");
    assert_eq!(query.node_type, "function_definition");
}

#[test]
fn test_javascript_query_patterns() {
    let query = AstQueryPatterns::undocumented_functions("javascript");
    assert_eq!(query.language, "javascript");
    assert_eq!(query.node_type, "function_declaration");
}

#[test]
fn test_go_query_patterns() {
    let query = AstQueryPatterns::undocumented_functions("go");
    assert_eq!(query.language, "go");
    assert_eq!(query.node_type, "function_declaration");
}
