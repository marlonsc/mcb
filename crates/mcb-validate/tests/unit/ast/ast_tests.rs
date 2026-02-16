//! Tests for AST analysis module.
//!
//! Uses `AST_ROOT_*` constants for expected root node kinds instead
//! of hardcoded strings. Removed redundant `#[cfg(test)] mod` wrapper.

use rstest::rstest;
use std::path::Path;

use rust_code_analysis::{Callback, LANG, ParserTrait, action, guess_language};

use crate::test_constants::{AST_ROOT_PROGRAM, AST_ROOT_PYTHON, AST_ROOT_RUST};

/// Simple callback that returns the root node kind.
struct RootKindCallback;

impl Callback for RootKindCallback {
    type Res = String;
    type Cfg = ();

    fn call<T: ParserTrait>(_cfg: Self::Cfg, parser: &T) -> Self::Res {
        parser.get_root().kind().to_owned()
    }
}

#[rstest]
#[case(b"fn main() {}", "main.rs", "rust")]
#[case(b"def main(): pass", "script.py", "python")]
#[case(b"function main() {}", "app.js", "javascript")]
#[case(b"function main(): void {}", "component.ts", "typescript")]
#[case(b"some content", "unknown.xyz", "unknown")]
fn language_detection(#[case] code: &[u8], #[case] file: &str, #[case] expected: &str) {
    let (lang, _) = guess_language(code, Path::new(file));

    match expected {
        "rust" => assert_eq!(lang, Some(LANG::Rust)),
        "python" => assert_eq!(lang, Some(LANG::Python)),
        "javascript" => assert!(matches!(lang, Some(LANG::Javascript | LANG::Mozjs))),
        "typescript" => assert!(matches!(lang, Some(LANG::Typescript | LANG::Tsx))),
        _ => assert_eq!(lang, None),
    }
}

#[rstest]
#[case(&LANG::Rust, b"fn hello_world() { println!(\"Hello\"); }", "test.rs", AST_ROOT_RUST)]
#[case(&LANG::Python, b"def hello(): print('Hello')", "test.py", AST_ROOT_PYTHON)]
#[case(&LANG::Mozjs, b"function hello() { console.log('Hello'); }", "test.js", AST_ROOT_PROGRAM)]
#[case(&LANG::Typescript, b"function hello(): void { console.log('Hello'); }", "test.ts", AST_ROOT_PROGRAM)]
fn parser_root_kind(
    #[case] lang: &LANG,
    #[case] code: &[u8],
    #[case] file: &str,
    #[case] expected_root: &str,
) {
    let path = Path::new(file);
    let root_kind = action::<RootKindCallback>(lang, code.to_vec(), path, None, ());
    assert_eq!(root_kind, expected_root);
}
