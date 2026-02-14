#![allow(unsafe_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use mcb_infrastructure::config::ConfigLoader;
use serial_test::serial;
use tempfile::TempDir;

struct CurrentDirGuard {
    original: PathBuf,
}

impl CurrentDirGuard {
    fn new(new_dir: &Path) -> Self {
        let original = env::current_dir().expect("read current dir");
        env::set_current_dir(new_dir).expect("set current dir");
        Self { original }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        env::set_current_dir(&self.original).expect("restore current dir");
    }
}

struct EnvVarGuard {
    key: String,
}

impl EnvVarGuard {
    fn set(key: &str, value: &str) -> Self {
        // SAFETY: These tests run under `#[serial]` so no concurrent env access.
        unsafe {
            env::set_var(key, value);
        }
        Self {
            key: key.to_string(),
        }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        // SAFETY: These tests run under `#[serial]` so no concurrent env access.
        unsafe {
            env::remove_var(&self.key);
        }
    }
}

struct RestoreFileGuard {
    backup: PathBuf,
    target: PathBuf,
}

impl RestoreFileGuard {
    fn move_out(target: &Path, backup: &Path) -> Self {
        fs::rename(target, backup).expect("move default.toml to backup");
        Self {
            backup: backup.to_path_buf(),
            target: target.to_path_buf(),
        }
    }
}

impl Drop for RestoreFileGuard {
    fn drop(&mut self) {
        if self.backup.exists() {
            fs::rename(&self.backup, &self.target).expect("restore default.toml from backup");
        }
    }
}

fn workspace_default_toml() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for dir in manifest_dir.ancestors() {
        let candidate = dir.join("config").join("default.toml");
        if candidate.exists() {
            return candidate;
        }
    }

    panic!("workspace config/default.toml not found from CARGO_MANIFEST_DIR ancestors");
}

fn write_temp_default(temp_dir: &TempDir, contents: &str) -> PathBuf {
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).expect("create temp config dir");
    let default_path = config_dir.join("default.toml");
    fs::write(&default_path, contents).expect("write temp default.toml");
    default_path
}

fn inject_key_into_section(toml: &str, section_header: &str, key_line: &str) -> String {
    let mut result = String::new();
    let mut found_section = false;
    let mut inserted = false;

    for line in toml.lines() {
        let trimmed = line.trim();

        if trimmed == section_header {
            found_section = true;
        } else if found_section && !inserted && trimmed.starts_with('[') {
            result.push_str(key_line);
            result.push('\n');
            inserted = true;
            found_section = false;
        }

        result.push_str(line);
        result.push('\n');
    }

    if found_section && !inserted {
        result.push_str(key_line);
        result.push('\n');
    }

    assert!(
        found_section || inserted,
        "section {section_header} not found in TOML"
    );

    result
}

fn remove_server_network_port(default_toml: &str) -> String {
    let mut out = String::new();
    let mut in_server_network = false;

    for line in default_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_server_network = trimmed == "[server.network]";
        }

        if in_server_network && trimmed.starts_with("port") && trimmed.contains('=') {
            continue;
        }

        out.push_str(line);
        out.push('\n');
    }

    out
}

#[test]
#[serial]
fn test_missing_default_toml_fails() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let _cwd_guard = CurrentDirGuard::new(temp_dir.path());

    let default_path = workspace_default_toml();
    let backup_path = default_path.with_extension("toml.strict-config-test-backup");
    let _restore = RestoreFileGuard::move_out(&default_path, &backup_path);

    let result = ConfigLoader::new().load();
    assert!(result.is_err(), "missing default.toml must fail");

    let message = result.expect_err("must fail").to_string();
    assert!(
        message.contains("default.toml")
            || message.contains("Default configuration file not found"),
        "error should mention missing default.toml, got: {message}"
    );
}

#[test]
#[serial]
fn test_unknown_key_rejected() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let default_toml =
        fs::read_to_string(workspace_default_toml()).expect("read workspace default.toml");

    // Inject a bogus key into the existing [server.network] section.
    // We must NOT duplicate the TOML section header — that would cause a
    // TOML parse error and give a false positive.
    let strict_toml =
        inject_key_into_section(&default_toml, "[server.network]", "bogus_key = true");
    let _default_path = write_temp_default(&temp_dir, &strict_toml);
    let _cwd_guard = CurrentDirGuard::new(temp_dir.path());

    let result = ConfigLoader::new().load();
    assert!(
        result.is_err(),
        "unknown keys in TOML should be rejected (strict mode / deny_unknown_fields), but load succeeded"
    );
}

#[test]
#[serial]
fn test_missing_required_key_fails() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let default_toml =
        fs::read_to_string(workspace_default_toml()).expect("read workspace default.toml");
    let missing_port_toml = remove_server_network_port(&default_toml);
    let _default_path = write_temp_default(&temp_dir, &missing_port_toml);
    let _cwd_guard = CurrentDirGuard::new(temp_dir.path());

    let result = ConfigLoader::new().load();
    assert!(
        result.is_err(),
        "missing required key server.network.port must fail"
    );

    let message = result.expect_err("must fail").to_string();
    assert!(
        message.contains("server.network.port") || message.contains("port"),
        "error should mention missing key, got: {message}"
    );
}

#[test]
#[serial]
fn test_mcp_env_override_port_works() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let default_toml =
        fs::read_to_string(workspace_default_toml()).expect("read workspace default.toml");
    let _default_path = write_temp_default(&temp_dir, &default_toml);
    let _cwd_guard = CurrentDirGuard::new(temp_dir.path());
    let _env_guard = EnvVarGuard::set("MCP__SERVER__NETWORK__PORT", "9999");

    let config = ConfigLoader::new()
        .load()
        .expect("load config with env override");
    assert_eq!(
        config.server.network.port, 9999,
        "MCP__SERVER__NETWORK__PORT should override port from default.toml"
    );
}

// ── Enforcement: no config bypass ────────────────────────────────────────────

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for dir in manifest_dir.ancestors() {
        if dir.join("Cargo.lock").exists() {
            return dir.to_path_buf();
        }
    }
    panic!("workspace root not found from CARGO_MANIFEST_DIR ancestors");
}

fn scan_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if !dir.exists() {
        return results;
    }
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs")
            && !path.to_string_lossy().contains("/tests/")
            && !path.to_string_lossy().contains("/test_")
        {
            results.push(path.to_path_buf());
        }
    }
    results
}

#[test]
fn test_no_direct_env_var_in_production_code() {
    let root = workspace_root();
    let allowed_paths: &[&str] = &[
        "mcb-infrastructure/src/config/loader.rs",
        "mcb-validate/src/config/",
    ];

    let banned_env_vars = [
        "MCB_EXECUTION_FLOW",
        "MCB_SESSION_ID",
        "MCB_SESSION_FILE",
        "MCB_PROJECT_ID",
    ];
    let mut violations = Vec::new();

    for crate_dir in &["mcb-server/src", "mcb-providers/src", "mcb-application/src"] {
        let dir = root.join("crates").join(crate_dir);
        for file_path in scan_rs_files(&dir) {
            let content = fs::read_to_string(&file_path).unwrap_or_default();
            let relative = file_path.strip_prefix(&root).unwrap_or(&file_path);

            let is_allowed = allowed_paths
                .iter()
                .any(|a| relative.to_string_lossy().contains(a));
            if is_allowed {
                continue;
            }

            for (line_num, line) in content.lines().enumerate() {
                for var in &banned_env_vars {
                    if line.contains(var) && line.contains("env::var") {
                        violations.push(format!(
                            "  {}:{}: {}",
                            relative.display(),
                            line_num + 1,
                            line.trim()
                        ));
                    }
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Direct env var reads found outside config loader (Task 3 / Task 5 must fix these):\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_no_impl_default_in_config_types() {
    let root = workspace_root();
    let types_dir = root.join("crates/mcb-infrastructure/src/config/types");

    let allowed = [
        "ServerConfigBuilder",
        "CacheProvider",
        "TransportMode",
        "OperatingMode",
        "PasswordAlgorithm",
        "EventBusProvider",
        "ServerSslConfig",
    ];

    let mut violations = Vec::new();

    for file_path in scan_rs_files(&types_dir) {
        let content = fs::read_to_string(&file_path).unwrap_or_default();
        let relative = file_path.strip_prefix(&root).unwrap_or(&file_path);

        for (line_num, line) in content.lines().enumerate() {
            if line.contains("impl Default for") {
                let is_allowed = allowed.iter().any(|a| line.contains(a));
                if !is_allowed {
                    violations.push(format!(
                        "  {}:{}: {}",
                        relative.display(),
                        line_num + 1,
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "impl Default found on operational config types (defaults must come from default.toml):\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_no_serde_default_in_config_types() {
    let root = workspace_root();
    let types_dir = root.join("crates/mcb-infrastructure/src/config/types");
    let mut violations = Vec::new();

    for file_path in scan_rs_files(&types_dir) {
        let content = fs::read_to_string(&file_path).unwrap_or_default();
        let relative = file_path.strip_prefix(&root).unwrap_or(&file_path);

        for (line_num, line) in content.lines().enumerate() {
            if line.contains("#[serde(default") {
                violations.push(format!(
                    "  {}:{}: {}",
                    relative.display(),
                    line_num + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "serde(default) found in config types (operational defaults must come from default.toml):\n{}",
        violations.join("\n")
    );
}
