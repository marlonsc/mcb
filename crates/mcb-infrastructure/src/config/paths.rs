//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#configuration)
//!
//! Centralized configuration path logic
//!
//! Provides standard locations for configuration files and directories.

use std::path::PathBuf;

use mcb_domain::error::{Error, Result};

/// Constant value for `VCS_REGISTRY_FILENAME`.
pub const VCS_REGISTRY_FILENAME: &str = "vcs_repository_registry.json";
/// Constant value for `VCS_LOCK_FILENAME`.
pub const VCS_LOCK_FILENAME: &str = "vcs_repository_registry.lock";
/// Constant value for `COLLECTION_MAPPING_FILENAME`.
pub const COLLECTION_MAPPING_FILENAME: &str = "collection_mapping.json";
/// Constant value for `COLLECTION_MAPPING_LOCK_FILENAME`.
pub const COLLECTION_MAPPING_LOCK_FILENAME: &str = "collection_mapping.lock";

/// Returns the main configuration directory for mcb (e.g., ~/.config/mcb)
///
/// # Errors
///
/// Returns an error if the system config directory cannot be determined.
pub fn config_dir() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| Error::config("Unable to determine config directory"))?;
    Ok(config_dir.join("mcb"))
}
