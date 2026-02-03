//! Unit tests for language module
//!
//! Tests for `LanguageId` and `LanguageRegistry` functionality.

use mcb_language_support::language::{LanguageId, LanguageRegistry};
use rust_code_analysis::LANG;

#[test]
fn test_language_id_all() {
    let all = LanguageId::all();
    assert_eq!(all.len(), 7);
    assert!(all.contains(&LanguageId::Rust));
    assert!(all.contains(&LanguageId::Kotlin));
}

#[test]
fn test_language_id_name() {
    assert_eq!(LanguageId::Rust.name(), "rust");
    assert_eq!(LanguageId::Cpp.name(), "cpp");
    assert_eq!(LanguageId::JavaScript.name(), "javascript");
}

#[test]
fn test_language_id_display_name() {
    assert_eq!(LanguageId::Cpp.display_name(), "C/C++");
    assert_eq!(LanguageId::JavaScript.display_name(), "JavaScript");
    assert_eq!(LanguageId::TypeScript.display_name(), "TypeScript");
}

#[test]
fn test_language_id_extensions() {
    assert!(LanguageId::Rust.extensions().contains(&"rs"));
    assert!(LanguageId::Python.extensions().contains(&"py"));
    assert!(LanguageId::JavaScript.extensions().contains(&"jsx"));
    assert!(LanguageId::Cpp.extensions().contains(&"c"));
    assert!(LanguageId::Cpp.extensions().contains(&"cpp"));
}

#[test]
fn test_language_id_from_name() {
    assert_eq!(LanguageId::from_name("rust"), Some(LanguageId::Rust));
    assert_eq!(LanguageId::from_name("PYTHON"), Some(LanguageId::Python));
    assert_eq!(LanguageId::from_name("c++"), Some(LanguageId::Cpp));
    assert_eq!(LanguageId::from_name("c"), Some(LanguageId::Cpp));
    assert_eq!(LanguageId::from_name("unknown"), None);
}

#[test]
fn test_language_id_rca_conversion() {
    assert_eq!(LanguageId::Rust.to_rca_lang(), LANG::Rust);
    assert_eq!(LanguageId::JavaScript.to_rca_lang(), LANG::Mozjs);

    assert_eq!(
        LanguageId::from_rca_lang(LANG::Rust),
        Some(LanguageId::Rust)
    );
    assert_eq!(
        LanguageId::from_rca_lang(LANG::Mozjs),
        Some(LanguageId::JavaScript)
    );
}

#[test]
fn test_language_registry_by_extension() {
    let registry = LanguageRegistry::new();

    assert_eq!(registry.by_extension("rs"), Some(LanguageId::Rust));
    assert_eq!(registry.by_extension(".py"), Some(LanguageId::Python));
    assert_eq!(registry.by_extension("JS"), Some(LanguageId::JavaScript));
    assert_eq!(registry.by_extension("unknown"), None);
}

#[test]
fn test_language_registry_info() {
    let registry = LanguageRegistry::new();

    let rust_info = registry
        .info(LanguageId::Rust)
        .expect("Rust should be registered");
    assert!(rust_info.supports_ast);
    assert_eq!(rust_info.comment_prefix, Some("//"));
    assert_eq!(rust_info.block_comment, Some(("/*", "*/")));

    let python_info = registry
        .info(LanguageId::Python)
        .expect("Python should be registered");
    assert_eq!(python_info.comment_prefix, Some("#"));
    assert_eq!(python_info.block_comment, None);
}

#[test]
fn test_language_registry_all_extensions() {
    let registry = LanguageRegistry::new();
    let extensions = registry.all_extensions();

    assert!(extensions.contains(&"rs"));
    assert!(extensions.contains(&"py"));
    assert!(extensions.contains(&"kt"));
}
