//! Shared test helpers for VCS tests.
//!
//! Re-exports git helpers from centralized `mcb_domain::utils::tests::git_helpers`.

use mcb_domain::ports::VcsProvider;
use mcb_domain::registry::vcs::{VcsProviderConfig, resolve_vcs_provider};
use mcb_domain::utils::tests::utils::TestResult;

// Re-export centralized git helpers
pub use mcb_domain::utils::tests::git_helpers::{create_test_repo, run_git};

pub fn vcs_provider() -> TestResult<std::sync::Arc<dyn VcsProvider>> {
    Ok(resolve_vcs_provider(&VcsProviderConfig::new("git"))?)
}
