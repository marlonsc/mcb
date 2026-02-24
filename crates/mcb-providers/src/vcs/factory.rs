//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
use std::sync::Arc;

use mcb_domain::ports::VcsProvider;

use super::git;

/// Builds the default VCS provider.
#[must_use]
pub fn default_vcs_provider() -> Arc<dyn VcsProvider> {
    Arc::new(git::GitProvider::new())
}
