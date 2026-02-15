//! Constants for documentation validation.

// === Regex Patterns ===

/// Regex for detecting doc comments (`///`).
pub const DOC_COMMENT_REGEX: &str = r"^\s*///";

/// Regex for capturing doc comment content after `///`.
pub const DOC_COMMENT_CAPTURE_REGEX: &str = r"^\s*///(.*)";

/// Regex for detecting attributes (`#[...]`).
pub const ATTR_REGEX: &str = r"^\s*#\[";

/// Regex for detecting module-level doc comments (`//!`).
pub const MODULE_DOC_REGEX: &str = "^//!";

/// Regex for detecting `pub struct` declarations.
pub const PUB_STRUCT_REGEX: &str = r"pub\s+struct\s+([A-Z][a-zA-Z0-9_]*)";

/// Regex for detecting `pub enum` declarations.
pub const PUB_ENUM_REGEX: &str = r"pub\s+enum\s+([A-Z][a-zA-Z0-9_]*)";

/// Regex for detecting `pub trait` declarations.
pub const PUB_TRAIT_REGEX: &str = r"pub\s+trait\s+([A-Z][a-zA-Z0-9_]*)";

/// Regex for detecting `pub fn` / `pub async fn` declarations.
pub const PUB_FN_REGEX: &str = r"pub\s+(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)";

/// Regex for detecting example sections in documentation.
pub const EXAMPLE_SECTION_REGEX: &str = r"#\s*Example";

// === Module File Names ===

/// File names that require module-level documentation.
pub const MODULE_FILE_NAMES: &[&str] = &["lib.rs", "mod.rs"];

// === Path Patterns for Skip Rules ===

/// Paths identifying DI module traits (skip example checking).
pub const DI_MODULES_PATH: &str = "/di/modules/";

/// Paths identifying port traits (skip example checking).
pub const PORTS_PATH: &str = "/ports/";

// === Item Kind Labels ===

/// Label for struct items in violation messages.
pub const ITEM_KIND_STRUCT: &str = "struct";

/// Label for enum items in violation messages.
pub const ITEM_KIND_ENUM: &str = "enum";

/// Label for trait items in violation messages.
pub const ITEM_KIND_TRAIT: &str = "trait";

/// Label for function items in violation messages.
pub const ITEM_KIND_FUNCTION: &str = "function";
