//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#dependency-injection)
//!
//! VCS provider factory for standalone/server composition.
//!
//! Provides a default VCS provider so that the server layer does not
//! import concrete providers directly (CA006).

use std::sync::Arc;

use mcb_domain::ports::VcsProvider;
use mcb_domain::registry::vcs::{VcsProviderConfig, resolve_vcs_provider};

/// Returns the default VCS provider for standalone and server modes.
///
/// Centralizes provider instantiation in the infrastructure layer.
///
/// # Errors
///
/// Returns an error if the configured/default VCS provider cannot be resolved.
pub fn default_vcs_provider() -> mcb_domain::error::Result<Arc<dyn VcsProvider>> {
    resolve_vcs_provider(&VcsProviderConfig::new("git2"))
}
