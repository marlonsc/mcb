//! Centralized configuration path logic
//!
//! Provides standard locations for configuration files and directories.

use std::path::PathBuf;

use mcb_domain::error::{Error, Result};

pub const VCS_REGISTRY_FILENAME: &str = "vcs_repository_registry.json";
pub const VCS_LOCK_FILENAME: &str = "vcs_repository_registry.lock";
pub const COLLECTION_MAPPING_FILENAME: &str = "collection_mapping.json";
pub const COLLECTION_MAPPING_LOCK_FILENAME: &str = "collection_mapping.lock";

/// Returns the main configuration directory for mcb (e.g., ~/.config/mcb)
pub fn config_dir() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| Error::config("Unable to determine config directory"))?;
    Ok(config_dir.join("mcb"))
}
