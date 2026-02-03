//! Tests for submodule entity (REF003: dedicated test file).

use mcb_domain::entities::submodule::{SubmoduleDiscoveryConfig, SubmoduleInfo};

#[test]
fn test_submodule_discovery_config_default() {
    let c = SubmoduleDiscoveryConfig::default();
    assert_eq!(c.max_depth, 2);
    assert!(!c.skip_uninitialized);
    assert!(c.continue_on_error);
}

#[test]
fn test_submodule_info_fields() {
    let s = SubmoduleInfo {
        id: "sub-1".to_string(),
        path: "libs/bar".to_string(),
        url: "https://example.com/bar.git".to_string(),
        commit_hash: "abc123".to_string(),
        parent_repo_id: "repo-1".to_string(),
        depth: 1,
        name: "bar".to_string(),
        is_initialized: true,
    };
    assert_eq!(s.id, "sub-1");
    assert_eq!(s.path, "libs/bar");
    assert_eq!(s.depth, 1);
}
