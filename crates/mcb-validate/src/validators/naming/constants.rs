//! Constants for naming validators.

// ============================================================================
// CA NAMING (checks/ca.rs)
// ============================================================================

/// Domain-layer file keywords that indicate port traits.
pub const CA_DOMAIN_PROVIDER_KEYWORD: &str = "provider";

/// Domain-layer file keyword for repositories.
pub const CA_DOMAIN_REPOSITORY_KEYWORD: &str = "repository";

/// Expected directory for provider ports.
pub const CA_PORTS_PROVIDERS_DIR: &str = "/ports/providers/";

/// Expected directory for ports (general).
pub const CA_PORTS_DIR: &str = "/ports/";

/// Expected directory for repositories.
pub const CA_REPOSITORIES_DIR: &str = "/repositories/";

/// Expected directory for repository adapters.
pub const CA_ADAPTERS_REPOSITORY_DIR: &str = "/adapters/repository/";

/// Infrastructure file name keywords for adapter files.
pub const CA_INFRA_IMPL_SUFFIX: &str = "_impl";

/// Infrastructure adapter file name keyword.
pub const CA_INFRA_ADAPTER_KEYWORD: &str = "adapter";

/// Expected directory for adapters.
pub const CA_ADAPTERS_DIR: &str = "/adapters/";

/// Infrastructure DI module file keyword.
pub const CA_MODULE_KEYWORD: &str = "module";

/// Expected directory for DI modules.
pub const CA_DI_DIR: &str = "/di/";

/// Server handler directories (allowed locations).
pub const CA_HANDLER_DIRS: &[&str] = &["/handlers/", "/admin/", "/tools/"];

/// Server handler file keyword.
pub const CA_HANDLER_KEYWORD: &str = "handler";

// ============================================================================
// MODULES (checks/modules.rs)
// ============================================================================

/// Special file names to skip in module naming checks.
pub const MODULE_SPECIAL_FILES: &[&str] = &["lib", "main", "build"];

/// Module file name.
pub const MODULE_FILE_NAME: &str = "mod";
