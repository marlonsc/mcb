//! Tests for AST analysis module
//!
//! These tests verify that:
//! - RCA `guess_language()` correctly detects languages
//! - RCA `action()` can parse code with custom Callbacks
//!
//! Using rust-code-analysis (RCA) directly - NO wrappers.

#[cfg(test)]
mod ast_tests {
    use std::path::Path;

    use rust_code_analysis::{Callback, LANG, ParserTrait, action, guess_language};

    /// Simple callback that returns the root node kind
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
        assert_eq!(root_kind, "source_file");
    }

    #[test]
    fn test_python_parser() {
        let code = b"def hello(): print('Hello')";
        let path = Path::new("test.py");

        let root_kind = action::<RootKindCallback>(&LANG::Python, code.to_vec(), path, None, ());
        assert_eq!(root_kind, "module");
    }

    #[test]
    fn test_javascript_parser() {
        let code = b"function hello() { console.log('Hello'); }";
        let path = Path::new("test.js");

        let root_kind = action::<RootKindCallback>(&LANG::Mozjs, code.to_vec(), path, None, ());
        assert_eq!(root_kind, "program");
    }

    #[test]
    fn test_typescript_parser() {
        let code = b"function hello(): void { console.log('Hello'); }";
        let path = Path::new("test.ts");

        let root_kind =
            action::<RootKindCallback>(&LANG::Typescript, code.to_vec(), path, None, ());
        assert_eq!(root_kind, "program");
    }
}
