//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Re-exports linter constants from the central constants module.
//!
//! All Clippy/Cargo constants live in [`crate::constants::linters`].

pub use crate::constants::linters::{
    CARGO_ARG_SEPARATOR,
    CARGO_TOML_FILENAME,
    CLIPPY_COMMAND,
    CLIPPY_MESSAGE_FORMAT_JSON,
    CLIPPY_PREFIX,
    CLIPPY_REASON_COMPILER_MESSAGE,
    CLIPPY_WARN_FLAG,
};
