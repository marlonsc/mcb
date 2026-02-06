//! Centralized configuration path logic
//!
//! Provides standard locations for configuration files and directories.

use mcb_domain::error::{Error, Result};
use std::path::PathBuf;

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

/// Returns the path to the VCS repository registry file
pub fn vcs_registry_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(VCS_REGISTRY_FILENAME))
}

/// Returns the path to the VCS repository registry lock file
pub fn vcs_registry_lock_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(VCS_LOCK_FILENAME))
}

/// Returns the path to the collection mapping file
pub fn collection_mapping_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(COLLECTION_MAPPING_FILENAME))
}

/// Returns the path to the collection mapping lock file
pub fn collection_mapping_lock_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(COLLECTION_MAPPING_LOCK_FILENAME))
}
