#![allow(unsafe_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use mcb_domain::error::{Error, Result as DomainResult};
use mcb_domain::test_fs_scan::scan_rs_files;
use mcb_domain::test_utils::workspace_root;
use mcb_infrastructure::config::{AppConfig, validate_app_config};
use rstest::rstest;
use tempfile::TempDir;

fn workspace_development_yaml() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for dir in manifest_dir.ancestors() {
        let candidate = dir.join("config").join("development.yaml");
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err("workspace config/development.yaml not found from CARGO_MANIFEST_DIR ancestors".into())
}

fn write_temp_yaml(
    temp_dir: &TempDir,
    contents: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir)?;
    let yaml_path = config_dir.join("development.yaml");
    fs::write(&yaml_path, contents)?;
    Ok(yaml_path)
}

fn inject_bogus_key_into_yaml(yaml: &str) -> String {
    let mut result = String::new();
    let mut found_settings = false;

    for line in yaml.lines() {
        result.push_str(line);
        result.push('\n');

        if line.trim() == "settings:" && !found_settings {
            found_settings = true;
            result.push_str("  bogus_key: true\n");
        }
    }

    result
}

fn remove_required_key_from_yaml(yaml: &str) -> String {
    let mut result = String::new();

    for line in yaml.lines() {
        let trimmed = line.trim();
        // password_algorithm is unique and required in auth config
        if trimmed.starts_with("password_algorithm:") {
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }

    result
}

fn load_app_config_from_yaml_path(path: &Path) -> DomainResult<AppConfig> {
    let content = fs::read_to_string(path)
        .map_err(|e| Error::config_with_source(format!("Failed to read {}", path.display()), e))?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| Error::config_with_source("Failed to parse YAML", e))?;
    let settings = yaml
        .get("settings")
        .ok_or_else(|| Error::ConfigMissing("No 'settings' key in config".to_owned()))?;
    let config: AppConfig = serde_yaml::from_value(settings.clone()).map_err(|e| {
        let detail = e.to_string();
        Error::config_with_source(format!("Failed to deserialize AppConfig: {detail}"), e)
    })?;
    validate_app_config(&config)?;
    Ok(config)
}

#[rstest]
#[test]
fn test_missing_yaml_config_fails() {
    let result =
        load_app_config_from_yaml_path(Path::new("/tmp/nonexistent-dir/config/development.yaml"));
    assert!(result.is_err(), "missing config file must fail");

    let message = result.expect_err("must fail").to_string();
    assert!(
        message.contains("Configuration file not found")
            || message.contains("not found")
            || message.contains("No such file or directory")
            || message.contains("Failed to read"),
        "error should mention missing file, got: {message}"
    );
}

#[rstest]
#[test]
fn test_unknown_key_rejected() {
    // AppConfig uses #[serde(deny_unknown_fields)] — unknown keys are rejected at parse time.
    let temp_dir = TempDir::new().unwrap();
    let yaml_content = fs::read_to_string(workspace_development_yaml().unwrap()).unwrap();
    let yaml_with_bogus = inject_bogus_key_into_yaml(&yaml_content);
    let yaml_path = write_temp_yaml(&temp_dir, &yaml_with_bogus).unwrap();
    let result = load_app_config_from_yaml_path(&yaml_path);
    assert!(
        result.is_err(),
        "unknown keys in YAML should be rejected (deny_unknown_fields), but load succeeded"
    );
}

#[rstest]
#[test]
fn test_missing_required_key_fails() {
    // Write a YAML config with a required key removed and load via explicit path.
    let temp_dir = TempDir::new().unwrap();
    let yaml_content = fs::read_to_string(workspace_development_yaml().unwrap()).unwrap();
    let missing_key_yaml = remove_required_key_from_yaml(&yaml_content);
    let yaml_path = write_temp_yaml(&temp_dir, &missing_key_yaml).unwrap();

    let result = load_app_config_from_yaml_path(&yaml_path);
    assert!(
        result.is_err(),
        "missing required key auth.password_algorithm must fail"
    );

    let message = result.expect_err("must fail").to_string();
    assert!(
        message.contains("password_algorithm") || message.contains("missing field"),
        "error should mention missing key, got: {message}"
    );
}

// NOTE: test_mcp_env_override_port_works was deleted because Figment (which supported
// MCP__ env var prefixes) was removed during the Figment→Loco YAML migration.
// Environment variable override is no longer supported in the YAML-based config system.
// If env override is needed in the future, it would need to be implemented as a separate

// ── Enforcement: no config bypass ────────────────────────────────────────────

#[rstest]
#[test]
fn test_no_direct_env_var_in_production_code() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let allowed_paths: &[&str] = &[
        "mcb-infrastructure/src/config/validation.rs",
        "mcb-validate/src/config/",
    ];

    let banned_env_vars = [
        "MCB_EXECUTION_FLOW",
        "MCB_SESSION_ID",
        "MCB_SESSION_FILE",
        "MCB_PROJECT_ID",
    ];
    let mut violations = Vec::new();

    for crate_dir in &[
        "mcb-server/src",
        "mcb-providers/src",
        "mcb-infrastructure/src/di",
    ] {
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
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("///") {
                    continue;
                }
                for var in &banned_env_vars {
                    if line.contains(var) {
                        violations.push(format!(
                            "  {}:{}: {}",
                            relative.display(),
                            line_num + 1,
                            trimmed
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
    Ok(())
}

#[rstest]
#[test]
fn test_no_impl_default_in_config_types() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let types_dir = root.join("crates/mcb-infrastructure/src/config/types");

    let allowed = [
        "CacheProvider",
        "OperatingMode",
        "PasswordAlgorithm",
        "EventBusBackend",
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
        "impl Default found on operational config types (defaults must come from development.yaml):\n{}",
        violations.join("\n")
    );
    Ok(())
}

#[rstest]
#[test]
fn test_no_serde_default_in_config_types() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
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
        "serde(default) found in config types (operational defaults must come from development.yaml):\n{}",
        violations.join("\n")
    );
    Ok(())
}

#[rstest]
#[test]
fn test_no_hardcoded_fallback_for_security_ids() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;

    let security_id_fallback_patterns: &[(&str, &str)] = &[
        (r#""default""#, "project_id"),
        (r#""default""#, "org_id"),
        (r#""default""#, "repo_id"),
        (r#""unknown""#, "project_id"),
    ];

    let scan_dirs = &[
        "mcb-server/src",
        "mcb-infrastructure/src/di",
        "mcb-providers/src",
    ];

    let mut violations = Vec::new();

    for crate_dir in scan_dirs {
        let dir = root.join("crates").join(crate_dir);
        for file_path in scan_rs_files(&dir) {
            let content = fs::read_to_string(&file_path).unwrap_or_default();
            let relative = file_path.strip_prefix(&root).unwrap_or(&file_path);

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("///") {
                    continue;
                }

                for (fallback_str, id_context) in security_id_fallback_patterns {
                    if line.contains(fallback_str) && line.contains(id_context) {
                        violations.push(format!(
                            "  {}:{}: hardcoded fallback {} near {}: {}",
                            relative.display(),
                            line_num + 1,
                            fallback_str,
                            id_context,
                            trimmed
                        ));
                    }
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Hardcoded fallback values for security-sensitive IDs (project_id, org_id, repo_id) \
         must never exist — auto-detect from workspace/MCP or fail:\n{}",
        violations.join("\n")
    );
    Ok(())
}

// ── Enforcement: no duplicated domain constants outside mcb-domain ───────────

#[rstest]
#[test]
fn test_no_lang_constants_outside_domain() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let domain_lang = root.join("crates/mcb-domain/src/constants/lang.rs");

    let scan_dirs = &[
        "mcb-server/src",
        "mcb-providers/src",
        "mcb-infrastructure/src",
    ];

    let mut violations = Vec::new();

    for crate_dir in scan_dirs {
        let dir = root.join("crates").join(crate_dir);
        for file_path in scan_rs_files(&dir) {
            let content = fs::read_to_string(&file_path).unwrap_or_default();
            let relative = file_path.strip_prefix(&root).unwrap_or(&file_path);

            if file_path == domain_lang {
                continue;
            }

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("///") {
                    continue;
                }
                if trimmed.contains("pub use mcb_domain::") {
                    continue;
                }
                if trimmed.starts_with("pub const LANG_")
                    && trimmed.contains("&str")
                    && !trimmed.contains("LANG_CHUNK_SIZE_MAP")
                {
                    violations.push(format!(
                        "  {}:{}: {}",
                        relative.display(),
                        line_num + 1,
                        trimmed
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "LANG_* constants must only be defined in mcb-domain/src/constants/lang.rs \
         (Single Source of Truth). Found duplicate definitions:\n{}",
        violations.join("\n")
    );
    Ok(())
}

#[rstest]
#[test]
fn test_no_bm25_constants_outside_domain() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;

    let scan_dirs = &[
        "mcb-server/src",
        "mcb-providers/src",
        "mcb-infrastructure/src",
    ];

    let bm25_patterns = &[
        "pub const HYBRID_SEARCH_BM25_K1",
        "pub const HYBRID_SEARCH_BM25_B",
        "pub const BM25_TOKEN_MIN_LENGTH",
        "pub const HYBRID_SEARCH_BM25_WEIGHT",
        "pub const HYBRID_SEARCH_SEMANTIC_WEIGHT",
        "pub const HYBRID_SEARCH_MAX_CANDIDATES",
        "pub const CONTENT_TYPE_JSON",
    ];

    let mut violations = Vec::new();

    for crate_dir in scan_dirs {
        let dir = root.join("crates").join(crate_dir);
        for file_path in scan_rs_files(&dir) {
            let content = fs::read_to_string(&file_path).unwrap_or_default();
            let relative = file_path.strip_prefix(&root).unwrap_or(&file_path);

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("///") {
                    continue;
                }
                if trimmed.contains("pub use mcb_domain::") {
                    continue;
                }
                for pattern in bm25_patterns {
                    if trimmed.starts_with(pattern) {
                        violations.push(format!(
                            "  {}:{}: {}",
                            relative.display(),
                            line_num + 1,
                            trimmed
                        ));
                    }
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "BM25/CONTENT_TYPE constants must only be defined in mcb-domain/src/constants/ \
         (Single Source of Truth). Found duplicate definitions:\n{}",
        violations.join("\n")
    );
    Ok(())
}

#[rstest]
#[test]
fn test_no_hardcoded_provider_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;

    let banned_patterns = &[
        "default_embedding_config",
        "default_vector_store_config",
        "default_cache_config",
        "default_language_config",
    ];

    let scan_dirs = &["mcb-infrastructure/src/di"];

    let mut violations = Vec::new();

    for crate_dir in scan_dirs {
        let dir = root.join("crates").join(crate_dir);
        for file_path in scan_rs_files(&dir) {
            let content = fs::read_to_string(&file_path).unwrap_or_default();
            let relative = file_path.strip_prefix(&root).unwrap_or(&file_path);

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("///") {
                    continue;
                }
                for pattern in banned_patterns {
                    if trimmed.contains(&format!("fn {pattern}")) {
                        violations.push(format!(
                            "  {}:{}: {}",
                            relative.display(),
                            line_num + 1,
                            trimmed
                        ));
                    }
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Hardcoded provider default functions must not exist in DI layer. \
         Provider defaults come from config/development.yaml:\n{}",
        violations.join("\n")
    );
    Ok(())
}

// ── Validation function coverage ─────────────────────────────────────────────

/// Load `AppConfig` from development.yaml WITHOUT running `validate_app_config`.
/// This lets us mutate fields and then call `validate_app_config` ourselves.
fn load_valid_config_unvalidated() -> AppConfig {
    let path = workspace_development_yaml().expect("development.yaml must exist");
    let content = fs::read_to_string(&path).expect("read development.yaml");
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content).expect("parse YAML");
    let settings = yaml.get("settings").expect("settings key");
    serde_yaml::from_value(settings.clone()).expect("deserialize AppConfig")
}

// ── Positive: valid development.yaml passes all 6 validators ────────────────

#[rstest]
#[test]
fn test_valid_development_config_passes_all_validators() {
    let config = load_valid_config_unvalidated();
    let result = validate_app_config(&config);
    assert!(
        result.is_ok(),
        "valid development.yaml must pass all validators, got: {:?}",
        result.err()
    );
}

// ── Negative: validate_auth_config ──────────────────────────────────────────

#[rstest]
#[case::empty_secret("\"\"", "JWT secret cannot be empty")]
#[case::short_secret("\"tooshort\"", "at least")]
#[test]
fn test_auth_enabled_with_invalid_secret_fails(
    #[case] secret_value: &str,
    #[case] expected_msg: &str,
) {
    let mut config = load_valid_config_unvalidated();
    config.auth.enabled = true;
    config.auth.jwt.secret = secret_value.trim_matches('"').to_owned();

    let result = validate_app_config(&config);
    assert!(
        result.is_err(),
        "auth validation must fail for secret={secret_value}"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains(expected_msg),
        "error should contain '{expected_msg}', got: {msg}"
    );
}

#[rstest]
#[test]
fn test_auth_disabled_skips_secret_validation() {
    let mut config = load_valid_config_unvalidated();
    config.auth.enabled = false;
    config.auth.jwt.secret = String::new(); // empty secret, but auth disabled
    assert!(
        validate_app_config(&config).is_ok(),
        "auth disabled should skip JWT secret validation"
    );
}

// ── Negative: validate_cache_config ─────────────────────────────────────────

#[rstest]
#[test]
fn test_cache_enabled_with_zero_ttl_fails() {
    let mut config = load_valid_config_unvalidated();
    config.system.infrastructure.cache.enabled = true;
    config.system.infrastructure.cache.default_ttl_secs = 0;

    let result = validate_app_config(&config);
    assert!(result.is_err(), "cache TTL=0 with cache enabled must fail");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Cache TTL cannot be 0"),
        "error should mention cache TTL, got: {msg}"
    );
}

#[rstest]
#[test]
fn test_cache_disabled_allows_zero_ttl() {
    let mut config = load_valid_config_unvalidated();
    config.system.infrastructure.cache.enabled = false;
    config.system.infrastructure.cache.default_ttl_secs = 0;
    assert!(
        validate_app_config(&config).is_ok(),
        "cache disabled should allow TTL=0"
    );
}

// ── Negative: validate_limits_config ────────────────────────────────────────

#[rstest]
#[case::zero_memory(0, 4, "Memory limit cannot be 0")]
#[case::zero_cpu(2147483648, 0, "CPU limit cannot be 0")]
#[test]
fn test_limits_zero_value_fails(
    #[case] memory: usize,
    #[case] cpu: usize,
    #[case] expected_msg: &str,
) {
    let mut config = load_valid_config_unvalidated();
    config.system.infrastructure.limits.memory_limit = memory;
    config.system.infrastructure.limits.cpu_limit = cpu;

    let result = validate_app_config(&config);
    assert!(
        result.is_err(),
        "limits validation must fail for memory={memory}, cpu={cpu}"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains(expected_msg),
        "error should contain '{expected_msg}', got: {msg}"
    );
}

#[rstest]
#[test]
fn test_limits_nonzero_passes() {
    let mut config = load_valid_config_unvalidated();
    config.system.infrastructure.limits.memory_limit = 1024;
    config.system.infrastructure.limits.cpu_limit = 1;
    assert!(
        validate_app_config(&config).is_ok(),
        "nonzero limits must pass validation"
    );
}

// ── Negative: validate_daemon_config ────────────────────────────────────────

#[rstest]
#[test]
fn test_daemon_enabled_with_zero_restart_attempts_fails() {
    let mut config = load_valid_config_unvalidated();
    config.operations_daemon.daemon.enabled = true;
    config.operations_daemon.daemon.max_restart_attempts = 0;

    let result = validate_app_config(&config);
    assert!(result.is_err(), "daemon with 0 restart attempts must fail");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("restart attempts cannot be 0"),
        "error should mention restart attempts, got: {msg}"
    );
}

#[rstest]
#[test]
fn test_daemon_disabled_allows_zero_restart_attempts() {
    let mut config = load_valid_config_unvalidated();
    config.operations_daemon.daemon.enabled = false;
    config.operations_daemon.daemon.max_restart_attempts = 0;
    assert!(
        validate_app_config(&config).is_ok(),
        "daemon disabled should allow 0 restart attempts"
    );
}

// ── Negative: validate_backup_config ────────────────────────────────────────

#[rstest]
#[test]
fn test_backup_enabled_with_zero_interval_fails() {
    let mut config = load_valid_config_unvalidated();
    config.system.data.backup.enabled = true;
    config.system.data.backup.interval_secs = 0;

    let result = validate_app_config(&config);
    assert!(result.is_err(), "backup with 0 interval must fail");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Backup interval cannot be 0"),
        "error should mention backup interval, got: {msg}"
    );
}

#[rstest]
#[test]
fn test_backup_disabled_allows_zero_interval() {
    let mut config = load_valid_config_unvalidated();
    config.system.data.backup.enabled = false;
    config.system.data.backup.interval_secs = 0;
    assert!(
        validate_app_config(&config).is_ok(),
        "backup disabled should allow 0 interval"
    );
}

// ── Negative: validate_operations_config ──────────────────────────────────

#[rstest]
#[case::zero_cleanup(0, 2592000, "cleanup interval cannot be 0")]
#[case::zero_retention(3600, 0, "retention period cannot be 0")]
#[test]
fn test_operations_tracking_enabled_with_zero_value_fails(
    #[case] cleanup: u64,
    #[case] retention: u64,
    #[case] expected_msg: &str,
) {
    let mut config = load_valid_config_unvalidated();
    config.operations_daemon.operations.tracking_enabled = true;
    config.operations_daemon.operations.cleanup_interval_secs = cleanup;
    config.operations_daemon.operations.retention_secs = retention;

    let result = validate_app_config(&config);
    assert!(
        result.is_err(),
        "operations tracking with cleanup={cleanup}, retention={retention} must fail"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains(expected_msg),
        "error should contain '{expected_msg}', got: {msg}"
    );
}

#[rstest]
#[test]
fn test_operations_tracking_disabled_allows_zero_values() {
    let mut config = load_valid_config_unvalidated();
    config.operations_daemon.operations.tracking_enabled = false;
    config.operations_daemon.operations.cleanup_interval_secs = 0;
    config.operations_daemon.operations.retention_secs = 0;
    assert!(
        validate_app_config(&config).is_ok(),
        "tracking disabled should allow zero cleanup/retention"
    );
}
