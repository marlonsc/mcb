//! Centralized regex patterns for validation
//!
//! All regex patterns are compiled once at startup using `once_cell::sync::Lazy`.
//! This avoids repeated compilation and eliminates `.unwrap()` calls on `Regex::new()`.

use once_cell::sync::Lazy;
use regex::Regex;

// =============================================================================
// Visibility Patterns
// =============================================================================

/// Matches public item declarations: `pub fn/struct/enum/type/const/static name`
pub static PUB_ITEM_FULL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^pub\s+(fn|struct|enum|type|const|static)\s+(\w+)").expect("Invalid regex")
});

/// Matches public item declarations without const/static
pub static PUB_ITEM: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^pub\s+(fn|struct|enum|type)\s+(\w+)").expect("Invalid regex"));

/// Matches pub(crate) visibility
pub static PUB_CRATE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^pub\(crate\)").expect("Invalid regex"));

// =============================================================================
// Config Quality Patterns
// =============================================================================

/// Matches Default trait implementations
pub static DEFAULT_IMPL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"impl\s+Default\s+for\s+(\w+)").expect("Invalid regex"));

/// Matches public struct declarations
pub static PUB_STRUCT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"pub\s+struct\s+(\w+)").expect("Invalid regex"));

/// Matches public field declarations
pub static PUB_FIELD: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"pub\s+(\w+):\s+").expect("Invalid regex"));

// =============================================================================
// Framework Patterns (Rocket/Axum)
// =============================================================================

/// Matches axum::Router usage
pub static AXUM_ROUTER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"axum::Router").expect("Invalid regex"));

/// Matches axum::routing usage
pub static AXUM_ROUTING: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"axum::routing::").expect("Invalid regex"));

/// Matches tower middleware usage
pub static TOWER_MIDDLEWARE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"tower(|_http)::").expect("Invalid regex"));

// =============================================================================
// Figment/Config Patterns
// =============================================================================

/// Matches Config::builder() calls
pub static CONFIG_BUILDER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"Config::builder\(\)").expect("Invalid regex"));

/// Matches config::Environment usage
pub static CONFIG_ENVIRONMENT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"config::Environment").expect("Invalid regex"));

/// Matches config::File usage
pub static CONFIG_FILE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"config::File").expect("Invalid regex"));

// =============================================================================
// Test Quality Patterns
// =============================================================================

/// Matches #[ignore] attribute
pub static IGNORE_ATTR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"#\[ignore\]").expect("Invalid regex"));

/// Matches #[test] or #[tokio::test] attribute
pub static TEST_ATTR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"#\[test\]|#\[tokio::test\]").expect("Invalid regex"));

/// Matches function declarations: `fn name`
pub static FN_DECL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"fn\s+(\w+)").expect("Invalid regex"));

/// Matches todo!() macro
pub static TODO_MACRO: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"todo!\(").expect("Invalid regex"));

/// Matches empty body: `{ }`
pub static EMPTY_BODY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{\s*\}").expect("Invalid regex"));

/// Matches stub assertions: assert!(true) or assert_eq!(true, true)
pub static STUB_ASSERT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"assert!\(true\)|assert_eq!\(true,\s*true\)").expect("Invalid regex"));

/// Matches doc comment start
pub static DOC_COMMENT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*///").expect("Invalid regex"));

/// Matches doc comment content
pub static DOC_COMMENT_CONTENT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*///(.*)").expect("Invalid regex"));

// =============================================================================
// Shaku/DI Patterns
// =============================================================================

/// Matches #[derive(Component)]
pub static COMPONENT_DERIVE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"#\[derive\(Component\)\]").expect("Invalid regex"));

/// Matches #[shaku(interface = ...)]
pub static SHAKU_INTERFACE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"#\[shaku\(interface\s*=\s*").expect("Invalid regex"));

/// Matches #[shaku(inject)]
pub static SHAKU_INJECT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"#\[shaku\(inject\)\]").expect("Invalid regex"));

/// Matches module! { macro
pub static MODULE_MACRO: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"module!\s*\{").expect("Invalid regex"));

/// Matches container.resolve()
pub static CONTAINER_RESOLVE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"container\.resolve\(\)").expect("Invalid regex"));

/// Matches use shaku imports
pub static SHAKU_IMPORT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"use\s+shaku").expect("Invalid regex"));

/// Matches Arc<dyn Trait>
pub static ARC_DYN_TRAIT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"Arc<dyn\s+\w+>").expect("Invalid regex"));

// =============================================================================
// Linkme/Inventory Patterns
// =============================================================================

/// Matches inventory::submit!
pub static INVENTORY_SUBMIT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"inventory::submit!").expect("Invalid regex"));

/// Matches inventory::collect!
pub static INVENTORY_COLLECT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"inventory::collect!").expect("Invalid regex"));

// =============================================================================
// Dependency Patterns
// =============================================================================

/// Matches use mcb_* imports
pub static USE_MCB: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"use\s+(mcb_[a-z_]+)").expect("Invalid regex"));

// =============================================================================
// Clean Architecture Patterns
// =============================================================================

/// Matches use mcb_providers
pub static USE_MCB_PROVIDERS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"use\s+mcb_providers(?:::|;)").expect("Invalid regex"));

/// Matches Service::new() calls
pub static SERVICE_NEW: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\w+Service)(?:Impl)?::new\s*\(").expect("Invalid regex"));

/// Matches use mcb_application::(services|use_cases)::*Impl
pub static USE_MCB_APPLICATION_IMPL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"use\s+mcb_application::(services|use_cases)::(\w+Impl)\b").expect("Invalid regex")
});

/// Matches use mcb_application::*::*Service
pub static USE_MCB_APPLICATION_SERVICE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"use\s+mcb_application::\w+::(\w+Service)(?:Impl)?\b").expect("Invalid regex")
});

/// Matches pub trait *Provider or *Service
pub static PUB_TRAIT_PROVIDER_SERVICE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*pub\s+trait\s+(\w+(?:Provider|Service))\s*(?::|where|\{)").expect("Invalid regex")
});

/// Matches pub use mcb_domain::
pub static PUB_USE_MCB_DOMAIN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"pub\s+use\s+mcb_domain::").expect("Invalid regex"));

/// Matches use mcb_application
pub static USE_MCB_APPLICATION: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"use\s+mcb_application(?:::|;)").expect("Invalid regex"));

/// Matches use mcb_application::path
pub static USE_MCB_APPLICATION_PATH: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"use\s+(mcb_application::\S+)").expect("Invalid regex"));

// =============================================================================
// Quality Patterns
// =============================================================================

/// Matches #[allow(dead_code)]
pub static ALLOW_DEAD_CODE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"#\[allow\(dead_code\)\]").expect("Invalid regex"));

/// Matches function declaration (pub or not)
pub static FN_DECL_FULL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:pub\s+)?fn\s+(\w+)").expect("Invalid regex"));

/// Matches field declaration (pub or not)
pub static FIELD_DECL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:pub\s+)?(\w+):\s+").expect("Invalid regex"));

/// Matches panic!() macro
pub static PANIC_MACRO: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"panic!\s*\(").expect("Invalid regex"));

/// Matches TODO/FIXME/XXX/HACK comments
pub static TODO_FIXME_COMMENT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(TODO|FIXME|XXX|HACK):?\s*(.*)").expect("Invalid regex"));

// =============================================================================
// Refactoring Patterns
// =============================================================================

/// Matches module declarations: `mod name;`
pub static MOD_DECL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:pub\s+)?mod\s+([a-z_][a-z0-9_]*)(?:\s*;)").expect("Invalid regex"));

// =============================================================================
// Test Organization Patterns
// =============================================================================

/// Matches #[cfg(test)]
pub static CFG_TEST: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"#\[cfg\(test\)\]").expect("Invalid regex"));

/// Matches mod tests {
pub static MOD_TESTS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"mod\s+tests\s*\{").expect("Invalid regex"));

/// Matches #[test] attribute
pub static TEST_ATTR_SIMPLE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"#\[test\]").expect("Invalid regex"));

/// Matches #[tokio::test] attribute
pub static TOKIO_TEST_ATTR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"#\[tokio::test\]").expect("Invalid regex"));

/// Matches async fn or fn declarations with name
pub static FN_DECL_WITH_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(").expect("Invalid regex"));

// =============================================================================
// Port/Adapter Patterns
// =============================================================================

/// Matches pub trait declarations
pub static PUB_TRAIT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"pub\s+trait\s+(\w+)").expect("Invalid regex"));

/// Matches async fn start
pub static ASYNC_FN_START: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:async\s+)?fn\s+\w+").expect("Invalid regex"));

// =============================================================================
// Performance Patterns
// =============================================================================

/// Matches loop start: for/while/loop
pub static LOOP_START: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(for|while|loop)\s+").expect("Invalid regex"));

/// Matches .clone() calls
pub static CLONE_CALL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\.clone\(\)").expect("Invalid regex"));

// =============================================================================
// Naming Patterns
// =============================================================================

/// Matches struct declarations with name
pub static STRUCT_DECL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:pub\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)").expect("Invalid regex"));

/// Matches enum declarations with name
pub static ENUM_DECL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:pub\s+)?enum\s+([A-Za-z_][A-Za-z0-9_]*)").expect("Invalid regex"));

/// Matches trait declarations with name
pub static TRAIT_DECL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:pub\s+)?trait\s+([A-Za-z_][A-Za-z0-9_]*)").expect("Invalid regex"));

/// Matches function declarations with generics or params
pub static FN_DECL_GENERIC: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]").expect("Invalid regex")
});

/// Matches const declarations
pub static CONST_DECL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:pub\s+)?const\s+([A-Za-z_][A-Za-z0-9_]*)\s*:").expect("Invalid regex"));

/// Matches static declarations
pub static STATIC_DECL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:pub\s+)?static\s+([A-Za-z_][A-Za-z0-9_]*)\s*:").expect("Invalid regex"));

// =============================================================================
// Documentation Patterns
// =============================================================================

/// Matches # Example in doc comments
pub static EXAMPLE_HEADER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"#\s*Example").expect("Invalid regex"));

/// Matches attribute start: #[
pub static ATTR_START: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*#\[").expect("Invalid regex"));

// =============================================================================
// Architecture Patterns
// =============================================================================

/// Matches Arc<ConcreteType>
pub static ARC_CONCRETE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"Arc<([A-Z][a-zA-Z0-9_]*)>").expect("Invalid regex"));

/// Matches pub trait with async fn and Send+Sync bounds
pub static ASYNC_TRAIT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"async\s+fn\s+").expect("Invalid regex"));

/// Matches Send + Sync bounds
pub static SEND_SYNC_BOUND: Lazy<Regex> =
    Lazy::new(|| Regex::new(r":\s*.*Send\s*\+\s*Sync").expect("Invalid regex"));

/// Matches #[async_trait] attribute
pub static ASYNC_TRAIT_ATTR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"#\[(async_trait::)?async_trait\]").expect("Invalid regex"));

/// Matches #[allow(async_fn_in_trait)]
pub static ALLOW_ASYNC_FN_TRAIT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"#\[allow\(async_fn_in_trait\)\]").expect("Invalid regex"));

// =============================================================================
// Async Patterns
// =============================================================================

/// Matches async fn declarations with name
pub static ASYNC_FN_DECL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"async\s+fn\s+(\w+)").expect("Invalid regex"));

/// Matches async fn (simple)
pub static ASYNC_FN_SIMPLE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"async\s+fn\s+").expect("Invalid regex"));

/// Matches async fn or .await
pub static ASYNC_INDICATOR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"async\s+fn|\.await").expect("Invalid regex"));

/// Matches tokio::spawn(
pub static TOKIO_SPAWN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"tokio::spawn\s*\(").expect("Invalid regex"));

/// Matches let x = tokio::spawn
pub static ASSIGNED_SPAWN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"let\s+\w+\s*=\s*tokio::spawn").expect("Invalid regex"));

/// Matches pub/async fn declarations with name
pub static PUB_ASYNC_FN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)").expect("Invalid regex"));

// =============================================================================
// SOLID Patterns
// =============================================================================

/// Matches impl block with optional generics: `impl<T> Type {` or `impl Type for Trait {`
pub static IMPL_BLOCK: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"impl(?:<[^>]*>)?\s+(\w+)\s*\{").expect("Invalid regex"));

/// Matches impl for trait: `impl Type for Trait`
pub static IMPL_FOR_TRAIT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"impl(?:<[^>]*>)?\s+([A-Z][a-zA-Z0-9_]*)\s+for\s+([A-Z][a-zA-Z0-9_]*)")
        .expect("Invalid regex")
});

/// Matches impl with full generics pattern
pub static IMPL_FULL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"impl(?:<[^>]*>)?\s+(?:([A-Z][a-zA-Z0-9_]*)|[A-Z][a-zA-Z0-9_]*\s+for\s+([A-Z][a-zA-Z0-9_]*))")
        .expect("Invalid regex")
});

/// Matches match keyword
pub static MATCH_KEYWORD: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bmatch\b").expect("Invalid regex"));

/// Matches arrow in match arms
pub static MATCH_ARROW: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"=>").expect("Invalid regex"));

/// Matches stub macros: panic!, todo!, unimplemented!
pub static STUB_MACROS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(panic!|todo!|unimplemented!)").expect("Invalid regex"));

/// Matches mutable self method: `fn name(&mut self`
pub static MUT_SELF_METHOD: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"fn\s+(\w+)\s*\(\s*&mut\s+self").expect("Invalid regex"));

// =============================================================================
// Clean Architecture Additional Patterns
// =============================================================================

/// Matches Provider::new() calls
pub static PROVIDER_NEW: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\w+Provider)(?:Impl)?::new\s*\(").expect("Invalid regex"));

/// Matches Repository::new() calls
pub static REPOSITORY_NEW: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\w+Repository)(?:Impl)?::new\s*\(").expect("Invalid regex"));

/// Matches public struct with opening brace
pub static PUB_STRUCT_WITH_BRACE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"pub\s+struct\s+(\w+)\s*\{").expect("Invalid regex"));

/// Matches ID field patterns: id:, uuid:, entity_id:
pub static ID_FIELD: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bid\s*:|uuid\s*:|entity_id\s*:").expect("Invalid regex"));

// =============================================================================
// Additional Common Patterns
// =============================================================================

/// Matches use statements with braces: `use path::{a, b}`
pub static USE_WITH_BRACES: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"use\s+[\w:]+::\{").expect("Invalid regex"));

/// Matches Result return type
pub static RESULT_RETURN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"->\s*Result<").expect("Invalid regex"));

/// Matches Option return type
pub static OPTION_RETURN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"->\s*Option<").expect("Invalid regex"));

/// Matches Vec allocation
pub static VEC_ALLOCATION: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"Vec::new\(\)|vec!\[").expect("Invalid regex"));

/// Matches String allocation
pub static STRING_ALLOCATION: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"String::new\(\)|\.to_string\(\)|\.to_owned\(\)").expect("Invalid regex"));

/// Matches Box allocation
pub static BOX_ALLOCATION: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"Box::new\(").expect("Invalid regex"));

/// Matches Arc/Mutex usage
pub static ARC_MUTEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"Arc<Mutex<|Mutex<Arc<").expect("Invalid regex"));

/// Matches unwrap() call
pub static UNWRAP_CALL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\.unwrap\(\)").expect("Invalid regex"));

/// Matches expect() call
pub static EXPECT_CALL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\.expect\(").expect("Invalid regex"))
