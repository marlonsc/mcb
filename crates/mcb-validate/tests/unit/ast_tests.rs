//! Tests for AST analysis module.
//!
//! Uses `AST_ROOT_*` constants for expected root node kinds instead
//! of hardcoded strings. Removed redundant `#[cfg(test)] mod` wrapper.

use std::path::Path;

use rust_code_analysis::{action, guess_language, Callback, ParserTrait, LANG};

use crate::test_constants::{AST_ROOT_PROGRAM, AST_ROOT_PYTHON, AST_ROOT_RUST};

/// Simple callback that returns the root node kind.
struct RootKindCallback;

impl Callback for RootKindCallback {
    type Res = String;
    type Cfg = ();

    fn call<T: ParserTrait>(_cfg: Self::Cfg, parser: &T) -> Self::Res {
        parser.get_root().kind().to_string()
    }
}

#[test]
fn test_language_detection() {
    // Rust
    let (lang, _) = guess_language(b"fn main() {}", Path::new("main.rs"));
    assert_eq!(lang, Some(LANG::Rust));

    // Python
    let (lang, _) = guess_language(b"def main(): pass", Path::new("script.py"));
    assert_eq!(lang, Some(LANG::Python));

    // JavaScript (detected as Mozjs in RCA)
    let (lang, _) = guess_language(b"function main() {}", Path::new("app.js"));
    assert!(matches!(lang, Some(LANG::Javascript | LANG::Mozjs)));

    // TypeScript
    let (lang, _) = guess_language(b"function main(): void {}", Path::new("component.ts"));
    assert!(matches!(lang, Some(LANG::Typescript | LANG::Tsx)));

    // Unknown
    let (lang, _) = guess_language(b"some content", Path::new("unknown.xyz"));
    assert_eq!(lang, None);
}

#[test]
fn test_rust_parser() {
    let code = b"fn hello_world() { println!(\"Hello\"); }";
    let path = Path::new("test.rs");

    let root_kind = action::<RootKindCallback>(&LANG::Rust, code.to_vec(), path, None, ());
    assert_eq!(root_kind, AST_ROOT_RUST);
}

#[test]
fn test_python_parser() {
    let code = b"def hello(): print('Hello')";
    let path = Path::new("test.py");

    let root_kind = action::<RootKindCallback>(&LANG::Python, code.to_vec(), path, None, ());
    assert_eq!(root_kind, AST_ROOT_PYTHON);
}

#[test]
fn test_javascript_parser() {
    let code = b"function hello() { console.log('Hello'); }";
    let path = Path::new("test.js");

    let root_kind = action::<RootKindCallback>(&LANG::Mozjs, code.to_vec(), path, None, ());
    assert_eq!(root_kind, AST_ROOT_PROGRAM);
}

#[test]
fn test_typescript_parser() {
    let code = b"function hello(): void { console.log('Hello'); }";
    let path = Path::new("test.ts");

    let root_kind = action::<RootKindCallback>(&LANG::Typescript, code.to_vec(), path, None, ());
    assert_eq!(root_kind, AST_ROOT_PROGRAM);
}
