//! Unit tests for MCP context config (`GitConfig`, `McpContextConfig`).

use mcb_infrastructure::config::{GitConfig, McpContextConfig};
use rstest::rstest;

#[rstest]
fn test_default_git_config() {
    let config = GitConfig::default();
    assert_eq!(config.branches, vec!["main", "HEAD", "current"]);
    assert_eq!(config.depth, 50);
    assert!(config.ignore_patterns.is_empty());
    assert!(config.include_submodules);
}

#[rstest]
fn test_parse_git_config_from_toml() {
    let toml_str = r#"
[git]
branches = ["main", "develop"]
depth = 100
ignore_patterns = ["*.log", "target/"]
include_submodules = false
"#;
    let config: McpContextConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.git.branches, vec!["main", "develop"]);
    assert_eq!(config.git.depth, 100);
    assert_eq!(config.git.ignore_patterns, vec!["*.log", "target/"]);
    assert!(!config.git.include_submodules);
}
