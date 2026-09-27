//! Tests pinning the path-containment invariant of `PatternRegistry::load_from_rules`.
//!
//! `MCB_RULES_DIR` is an operator environment override, so the rules directory
//! is a trust boundary: a rule file that reaches it through a symlink (or any
//! other path resolving outside the canonical directory) must never be loaded.

use std::fs;
use std::path::Path;

use mcb_validate::pattern_registry::PatternRegistry;
use tempfile::TempDir;

fn naming_config() -> mcb_validate::config::NamingRulesConfig {
    mcb_validate::config::NamingRulesConfig {
        enabled: false,
        server_crate: "mcb-server".to_owned(),
        domain_crate: "mcb-domain".to_owned(),
        infrastructure_crate: "mcb-infrastructure".to_owned(),
        application_crate: String::new(),
        providers_crate: "mcb-providers".to_owned(),
        validate_crate: "mcb-validate".to_owned(),
        utils_crate: "mcb-utils".to_owned(),
    }
}

fn write_rule(dir: &Path, file: &str, id: &str, marker: &str) -> std::io::Result<()> {
    fs::write(
        dir.join(file),
        format!("id: \"{id}\"\npatterns:\n  marker: '{marker}'\n"),
    )
}

#[test]
fn rule_file_symlinked_outside_rules_dir_is_not_loaded() -> std::io::Result<()> {
    let outside = TempDir::new()?;
    let rules = TempDir::new()?;

    write_rule(outside.path(), "outside.yml", "ESC001", "escaped_marker")?;
    write_rule(rules.path(), "inside.yml", "INS001", "contained_marker")?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        outside.path().join("outside.yml"),
        rules.path().join("escape.yml"),
    )?;

    let registry = PatternRegistry::load_from_rules(rules.path(), &naming_config(), "mcb")
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    assert!(
        registry.contains("INS001.marker"),
        "contained rule file must load"
    );
    assert!(
        !registry.contains("ESC001.marker"),
        "rule file resolving outside the rules directory must be skipped"
    );
    Ok(())
}
