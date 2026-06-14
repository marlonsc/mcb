//! Tests pinning the embedded-defaults build invariant of `FileConfig`.
//!
//! `FileConfig::load` falls back to the binary-embedded
//! `config/mcb-validate.toml` (layer 1) and aborts via `unreachable!` if that
//! compile-time constant fails to parse. These tests prove the invariant holds,
//! so the panic path is unreachable in practice and a malformed embedded TOML
//! is caught in CI instead of at runtime.

use std::path::PathBuf;

use mcb_validate::config::FileConfig;
use tempfile::TempDir;

#[test]
fn embedded_defaults_parse_into_file_config() {
    let temp = TempDir::new().expect("tempdir");

    // load() exercises the embedded (layer-1) defaults; a malformed embedded
    // TOML would hit the `unreachable!` in load() and panic here.
    let config = FileConfig::load(temp.path());

    // Values sourced from the embedded config/mcb-validate.toml prove the
    // constant actually deserialized (not an empty/default struct).
    assert!(
        config
            .general
            .exclude_patterns
            .iter()
            .any(|p| p == "target/"),
        "embedded defaults must include the target/ exclude pattern"
    );
    assert_eq!(config.general.output_format, "human");
    assert_eq!(config.general.rules_path, PathBuf::from("rules"));
}

#[test]
fn load_sets_workspace_root_to_given_path() {
    let temp = TempDir::new().expect("tempdir");
    let config = FileConfig::load(temp.path());

    assert_eq!(config.general.workspace_root.as_deref(), Some(temp.path()));
}
