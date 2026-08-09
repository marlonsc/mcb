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
    let _config = FileConfig::load(temp.path());
}

#[test]
fn load_sets_workspace_root_to_given_path() {
    let temp = TempDir::new().expect("tempdir");
    let config = FileConfig::load(temp.path());

    assert_eq!(config.general.workspace_root.as_deref(), Some(temp.path()));
}
