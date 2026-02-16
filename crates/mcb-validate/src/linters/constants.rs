//! Constants for the linter integration module.

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
