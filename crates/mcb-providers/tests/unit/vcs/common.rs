//! Shared test helpers for VCS tests.
//!
//! Provides VCS provider resolution for tests.

use mcb_domain::ports::VcsProvider;
use mcb_domain::registry::vcs::{VcsProviderConfig, resolve_vcs_provider};
use mcb_domain::utils::tests::utils::TestResult;

pub fn vcs_provider() -> TestResult<std::sync::Arc<dyn VcsProvider>> {
    Ok(resolve_vcs_provider(&VcsProviderConfig::new(
        mcb_utils::constants::DEFAULT_VCS_PROVIDER,
    ))?)
}
