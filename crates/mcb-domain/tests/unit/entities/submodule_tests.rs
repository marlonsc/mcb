//! Tests for submodule entity (REF003: dedicated test file).

use mcb_domain::entities::submodule::{SubmoduleDiscoveryConfig, SubmoduleInfo};
use rstest::rstest;

#[rstest]
fn test_submodule_discovery_config_default() {
    let c = SubmoduleDiscoveryConfig::default();
    assert_eq!(c.max_depth, 2);
    assert!(!c.skip_uninitialized);
    assert!(c.continue_on_error);
}

#[rstest]
fn test_submodule_info_fields() {
    let s = SubmoduleInfo {
        id: "sub-1".to_owned(),
        path: "libs/bar".to_owned(),
        url: "https://example.com/bar.git".to_owned(),
        commit_hash: "abc123".to_owned(),
        parent_repo_id: "repo-1".to_owned(),
        depth: 1,
        name: "bar".to_owned(),
        is_initialized: true,
    };
    assert_eq!(s.id, "sub-1");
    assert_eq!(s.path, "libs/bar");
    assert_eq!(s.depth, 1);
}
