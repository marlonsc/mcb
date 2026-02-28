//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#configuration)
//!
//! Admin configuration path resolution for sea-orm-pro admin panel.
//!
//! Provides path resolution for admin configuration without direct dependency
//! on sea-orm-pro (which is a third-party, non-integrated crate).

use std::path::PathBuf;

use mcb_domain::error::Result;

/// Resolves the admin configuration root directory.
///
/// Returns the path to the admin configuration directory, defaulting to
/// `config/pro_admin` relative to the application root.
///
/// # Errors
///
/// Returns an error if the configuration directory cannot be determined.
pub fn resolve_admin_config_root() -> Result<PathBuf> {
    // Use the standard config directory pattern
    // In production, this would be relative to the app root
    // For now, we use a simple relative path that works in development
    Ok(PathBuf::from("config/pro_admin"))
}
