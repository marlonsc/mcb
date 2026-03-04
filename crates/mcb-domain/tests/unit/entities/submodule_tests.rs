//! Tests for submodule entity (REF003: dedicated test file).

use mcb_domain::entities::submodule::{SubmoduleDiscoveryConfig, SubmoduleInfo};
use rstest::{fixture, rstest};

#[fixture]
fn default_submodule_discovery_config() -> SubmoduleDiscoveryConfig {
    SubmoduleDiscoveryConfig::default()
}

#[rstest]
fn test_submodule_discovery_config_default(
    default_submodule_discovery_config: SubmoduleDiscoveryConfig,
) {
    assert_eq!(default_submodule_discovery_config.max_depth, 2);
    assert!(!default_submodule_discovery_config.skip_uninitialized);
    assert!(default_submodule_discovery_config.continue_on_error);
}

#[fixture]
fn default_submodule_info() -> SubmoduleInfo {
    SubmoduleInfo {
        id: "sub-1".to_owned(),
        path: "libs/bar".to_owned(),
        url: "https://example.com/bar.git".to_owned(),
        commit_hash: "abc123".to_owned(),
        parent_repo_id: "repo-1".to_owned(),
        depth: 1,
        name: "bar".to_owned(),
        is_initialized: true,
    }
}

#[rstest]
fn test_submodule_info_fields(default_submodule_info: SubmoduleInfo) {
    assert_eq!(default_submodule_info.id, "sub-1");
    assert_eq!(default_submodule_info.path, "libs/bar");
    assert_eq!(default_submodule_info.depth, 1);
}
