//! Architecture path fragments, clean architecture naming, and linter integration.

// ============================================================================
// Architecture Path Fragments
// ============================================================================

/// Path fragment identifying the handlers directory.
pub const ARCH_PATH_HANDLERS: &str = "/handlers/";

/// Path fragment identifying the services directory.
pub const ARCH_PATH_SERVICES: &str = "/services/";

/// Path fragment identifying the domain layer.
pub const ARCH_PATH_DOMAIN: &str = "/domain/";

/// Path fragment identifying the adapters directory.
pub const ADAPTERS_DIR: &str = "/adapters/";

/// Path fragment identifying the config directory.
pub const ARCH_PATH_CONFIG: &str = "/config/";

// ============================================================================
// Clean Architecture Naming and Layout
// ============================================================================

/// Domain-layer file keywords that indicate port traits.
pub const CA_DOMAIN_PROVIDER_KEYWORD: &str = "provider";

/// Domain-layer file keyword for repositories.
pub const CA_DOMAIN_REPOSITORY_KEYWORD: &str = "repository";

/// Expected directory for provider ports.
pub const CA_PORTS_PROVIDERS_DIR: &str = "/ports/providers/";

/// Expected directory for ports (general).
pub const PORTS_DIR: &str = "/ports/";

/// Expected directory for repositories.
pub const CA_REPOSITORIES_DIR: &str = "/repositories/";

/// Expected directory for repository adapters.
pub const CA_ADAPTERS_REPOSITORY_DIR: &str = "/adapters/repository/";

/// Infrastructure file name keywords for adapter files.
pub const CA_INFRA_IMPL_SUFFIX: &str = "_impl";

/// Infrastructure adapter file name keyword.
pub const CA_INFRA_ADAPTER_KEYWORD: &str = "adapter";

/// Infrastructure DI module file keyword.
pub const CA_MODULE_KEYWORD: &str = "module";

/// Expected directory for DI modules.
pub const CA_DI_DIR: &str = "/di/";

/// Server handler directories (allowed locations).
pub const CA_HANDLER_DIRS: &[&str] = &["/handlers/", "/admin/", "/tools/"];

/// Server handler file keyword.
pub const CA_HANDLER_KEYWORD: &str = "handler";

/// Special file names to skip in module naming checks.
pub const MODULE_SPECIAL_FILES: &[&str] = &["lib", "main", "build"];

/// Module file name (file stem).
pub const MODULE_FILE_NAME: &str = "mod";

// ============================================================================
// Labels (Pending-task and Stub Detection)
// ============================================================================

/// Label for pending task comments (first priority).
pub const PENDING_LABEL_TODO: &str = concat!("TO", "DO");

/// Label for fix-needed comments.
pub const PENDING_LABEL_FIXME: &str = concat!("FI", "XME");

/// Label for attention-needed comments.
pub const PENDING_LABEL_XXX: &str = concat!("X", "XX");

/// Label for workaround/shortcut comments.
pub const PENDING_LABEL_HACK: &str = concat!("HA", "CK");

/// Label for panic-stub detection (unimplemented placeholders).
pub const STUB_PANIC_LABEL: &str = concat!("panic(", "TO", "DO)");

// ============================================================================
// Linter Integration (Clippy / Cargo)
// ============================================================================

/// Clippy rule code prefix.
pub const CLIPPY_PREFIX: &str = "clippy::";

/// Clippy CLI warning flag.
pub const CLIPPY_WARN_FLAG: &str = "-W";

/// Clippy subcommand name.
pub const CLIPPY_COMMAND: &str = "clippy";

/// Clippy JSON output format flag.
pub const CLIPPY_MESSAGE_FORMAT_JSON: &str = "--message-format=json";

/// Cargo argument separator.
pub const CARGO_ARG_SEPARATOR: &str = "--";

/// Cargo manifest filename.
pub const CARGO_TOML_FILENAME: &str = "Cargo.toml";

/// Clippy compiler-message reason string.
pub const CLIPPY_REASON_COMPILER_MESSAGE: &str = "compiler-message";
