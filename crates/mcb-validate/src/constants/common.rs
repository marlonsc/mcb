//! Common validation constants shared across multiple validators.

/// Marker for test module configuration attribute.
pub const CFG_TEST_MARKER: &str = "#[cfg(test)]";

/// Line comment prefix.
pub const COMMENT_PREFIX: &str = "//";

/// Doc comment prefix.
pub const DOC_COMMENT_PREFIX: &str = "///";

/// Module-level doc comment prefix.
pub const MODULE_DOC_PREFIX: &str = "//!";

/// Attribute macro prefix.
pub const ATTRIBUTE_PREFIX: &str = "#[";

/// Line prefixes that indicate a const or static declaration.
pub const CONST_DECLARATION_PREFIXES: &[&str] = &["const ", "pub const ", "static ", "pub static "];

/// Path fragments that indicate a test file or directory.
pub const TEST_PATH_PATTERNS: &[&str] = &["/tests/", "/target/", "_test.rs", "test.rs"];

/// File stems that should be skipped in many validators.
pub const STANDARD_SKIP_FILES: &[&str] = &["lib", "mod", "main", "build"];

/// File name keywords that identify a constants file (skip in magic number checks, etc.).
pub const CONSTANTS_FILE_KEYWORDS: &[&str] = &["constant", "config"];

/// Prefix for test function names.
pub const TEST_FUNCTION_PREFIX: &str = "test_";

// ============================================================================
// Architecture Path Fragments (shared by organization, naming, documentation validators)
// ============================================================================

/// Path fragment identifying the ports directory.
pub const ARCH_PATH_PORTS: &str = "/ports/";

/// Path fragment identifying the ports/providers directory.
pub const ARCH_PATH_PORTS_PROVIDERS: &str = "/ports/providers/";

/// Path fragment identifying the DI directory.
pub const ARCH_PATH_DI: &str = "/di/";

/// Path fragment identifying the DI modules directory.
pub const ARCH_PATH_DI_MODULES: &str = "/di/modules/";

/// Path fragment identifying the handlers directory.
pub const ARCH_PATH_HANDLERS: &str = "/handlers/";

/// Path fragment identifying the admin directory.
pub const ARCH_PATH_ADMIN: &str = "/admin/";

/// Path fragment identifying the server layer.
pub const ARCH_PATH_SERVER: &str = "/server/";

/// Path fragment identifying the application layer.
pub const ARCH_PATH_APPLICATION: &str = "/application/";

/// Path fragment identifying the infrastructure layer.
pub const ARCH_PATH_INFRASTRUCTURE: &str = "/infrastructure/";
