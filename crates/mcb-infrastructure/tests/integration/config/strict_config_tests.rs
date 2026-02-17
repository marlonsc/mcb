#![allow(unsafe_code)]

use std::env;
use std::fs;
use std::path::PathBuf;

use mcb_infrastructure::config::ConfigLoader;
use serial_test::serial;
use tempfile::TempDir;

use crate::utils::env_vars::EnvVarGuard;
use crate::utils::fs_guards::{CurrentDirGuard, RestoreFileGuard};
use crate::utils::workspace::{scan_rs_files, workspace_root};

fn workspace_default_toml() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for dir in manifest_dir.ancestors() {
        let candidate = dir.join("config").join("default.toml");
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err("workspace config/default.toml not found from CARGO_MANIFEST_DIR ancestors".into())
}

fn write_temp_default(
    temp_dir: &TempDir,
    contents: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir)?;
    let default_path = config_dir.join("default.toml");
    fs::write(&default_path, contents)?;
    Ok(default_path)
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
fn test_missing_default_toml_fails() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let _cwd_guard = CurrentDirGuard::new(temp_dir.path())?;

    let default_path = workspace_default_toml()?;
    let backup_path = default_path.with_extension("toml.strict-config-test-backup");
    let _restore = RestoreFileGuard::move_out(&default_path, &backup_path)?;

    let result = ConfigLoader::new().load();
    assert!(result.is_err(), "missing default.toml must fail");

    let message = result.expect_err("must fail").to_string();
    assert!(
        message.contains("default.toml")
            || message.contains("Default configuration file not found"),
        "error should mention missing default.toml, got: {message}"
    );
    Ok(())
}

#[test]
#[serial]
fn test_unknown_key_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let default_toml = fs::read_to_string(workspace_default_toml()?)?;

    // Inject a bogus key into the existing [server.network] section.
    // We must NOT duplicate the TOML section header — that would cause a
    // TOML parse error and give a false positive.
    let strict_toml =
        inject_key_into_section(&default_toml, "[server.network]", "bogus_key = true");
    let _default_path = write_temp_default(&temp_dir, &strict_toml)?;
    let _cwd_guard = CurrentDirGuard::new(temp_dir.path())?;

    let result = ConfigLoader::new().load();
    assert!(
        result.is_err(),
        "unknown keys in TOML should be rejected (strict mode / deny_unknown_fields), but load succeeded"
    );
    Ok(())
}

#[test]
#[serial]
fn test_missing_required_key_fails() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let default_toml = fs::read_to_string(workspace_default_toml()?)?;
    let missing_port_toml = remove_server_network_port(&default_toml);
    let _default_path = write_temp_default(&temp_dir, &missing_port_toml)?;
    let _cwd_guard = CurrentDirGuard::new(temp_dir.path())?;

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
    Ok(())
}

#[test]
#[serial]
fn test_mcp_env_override_port_works() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let default_toml = fs::read_to_string(workspace_default_toml()?)?;
    let _default_path = write_temp_default(&temp_dir, &default_toml)?;
    let _cwd_guard = CurrentDirGuard::new(temp_dir.path())?;
    let _env_guard = EnvVarGuard::set("MCP__SERVER__NETWORK__PORT", "9999");

    let config = ConfigLoader::new().load()?;
    assert_eq!(
        config.server.network.port, 9999,
        "MCP__SERVER__NETWORK__PORT should override port from default.toml"
    );
    Ok(())
}

// ── Enforcement: no config bypass ────────────────────────────────────────────

#[test]
fn test_no_direct_env_var_in_production_code() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
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

    for crate_dir in &[
        "mcb-server/src",
        "mcb-providers/src",
        "mcb-application/src",
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

#[test]
fn test_no_impl_default_in_config_types() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
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
    Ok(())
}

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
        "serde(default) found in config types (operational defaults must come from default.toml):\n{}",
        violations.join("\n")
    );
    Ok(())
}

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

#[test]
fn test_no_lang_constants_outside_domain() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let domain_lang = root.join("crates/mcb-domain/src/constants/lang.rs");

    // Only scan non-domain crate source dirs for LANG_ constant definitions
    let scan_dirs = &[
        "mcb-server/src",
        "mcb-providers/src",
        "mcb-application/src",
        "mcb-infrastructure/src",
    ];

    let mut violations = Vec::new();

    for crate_dir in scan_dirs {
        let dir = root.join("crates").join(crate_dir);
        for file_path in scan_rs_files(&dir) {
            // Skip re-export files (allowed to `pub use mcb_domain::...`)
            let content = fs::read_to_string(&file_path).unwrap_or_default();
            let relative = file_path.strip_prefix(&root).unwrap_or(&file_path);

            // Skip the canonical definition
            if file_path == domain_lang {
                continue;
            }

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("///") {
                    continue;
                }
                // Allow re-exports from domain
                if trimmed.contains("pub use mcb_domain::") {
                    continue;
                }
                // Detect local definitions of LANG_* string identifier constants
                // Allow LANG_CHUNK_SIZE_MAP (provider-specific mapping table, not an identifier)
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

#[test]
fn test_no_bm25_constants_outside_domain() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;

    let scan_dirs = &[
        "mcb-server/src",
        "mcb-providers/src",
        "mcb-application/src",
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
                // Allow re-exports from domain
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

#[test]
fn test_no_hardcoded_provider_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;

    // These function names indicate hardcoded fallback defaults for providers
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
         Provider defaults come from config/default.toml:\n{}",
        violations.join("\n")
    );
    Ok(())
}
