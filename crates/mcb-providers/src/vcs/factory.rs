//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
use std::sync::Arc;

use mcb_domain::ports::VcsProvider;

use super::git2_provider;

/// Builds the default VCS provider implementation.
#[must_use]
pub fn default_vcs_provider() -> Arc<dyn VcsProvider> {
    Arc::new(git2_provider::Git2Provider::new())
}
