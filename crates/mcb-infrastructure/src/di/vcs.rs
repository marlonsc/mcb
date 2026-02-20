//! VCS provider factory for standalone/server composition.
//!
//! Provides a default VCS provider so that the server layer does not
//! import concrete providers directly (CA006).

use std::sync::Arc;

use mcb_domain::ports::VcsProvider;
use mcb_providers::vcs;

/// Returns the default VCS provider for standalone and server modes.
///
/// Centralizes provider instantiation in the infrastructure layer.
#[must_use]
pub fn default_vcs_provider() -> Arc<dyn VcsProvider> {
    vcs::default_vcs_provider()
}
