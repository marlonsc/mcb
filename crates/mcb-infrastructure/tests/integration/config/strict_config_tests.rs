#![allow(unsafe_code)]

use std::env;
use std::fs;
use std::path::PathBuf;

use crate::utils::workspace::{scan_rs_files, workspace_root};
use mcb_infrastructure::config::ConfigLoader;
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

fn remove_port_from_yaml(yaml: &str) -> String {
    let mut result = String::new();

    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("port:") {
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }

    result
}

#[test]
fn test_missing_yaml_config_fails() {
    // Use with_config_path pointing to a non-existent file — bypasses CWD/CARGO_MANIFEST_DIR
    // fallback, directly testing the missing-file error path.
    let result = ConfigLoader::new()
        .with_config_path("/tmp/nonexistent-dir/config/development.yaml")
        .load();
    assert!(result.is_err(), "missing config file must fail");

    let message = result.expect_err("must fail").to_string();
    assert!(
        message.contains("Configuration file not found") || message.contains("not found"),
        "error should mention missing file, got: {message}"
    );
}

#[test]
fn test_unknown_key_rejected() {
    // AppConfig uses #[serde(deny_unknown_fields)] — unknown keys are rejected at parse time.
    let temp_dir = TempDir::new().unwrap();
    let yaml_content = fs::read_to_string(workspace_development_yaml().unwrap()).unwrap();
    let yaml_with_bogus = inject_bogus_key_into_yaml(&yaml_content);
    let yaml_path = write_temp_yaml(&temp_dir, &yaml_with_bogus).unwrap();
    let result = ConfigLoader::new().with_config_path(&yaml_path).load();
    assert!(
        result.is_err(),
        "unknown keys in YAML should be rejected (deny_unknown_fields), but load succeeded"
    );
}

#[test]
fn test_missing_required_key_fails() {
    // Write a YAML config with the port key removed and load via explicit path.
    let temp_dir = TempDir::new().unwrap();
    let yaml_content = fs::read_to_string(workspace_development_yaml().unwrap()).unwrap();
    let missing_port_yaml = remove_port_from_yaml(&yaml_content);
    let yaml_path = write_temp_yaml(&temp_dir, &missing_port_yaml).unwrap();

    let result = ConfigLoader::new().with_config_path(&yaml_path).load();
    assert!(
        result.is_err(),
        "missing required key server.network.port must fail"
    );

    let message = result.expect_err("must fail").to_string();
    assert!(
        message.contains("port") || message.contains("missing field"),
        "error should mention missing key, got: {message}"
    );
}

// NOTE: test_mcp_env_override_port_works was deleted because Figment (which supported
// MCP__ env var prefixes) was removed during the Figment→Loco YAML migration.
// Environment variable override is no longer supported in the YAML-based config system.
// If env override is needed in the future, it would need to be implemented as a separate
// feature in ConfigLoader using serde_yaml's environment variable support.

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
        "impl Default found on operational config types (defaults must come from development.yaml):\n{}",
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
        "serde(default) found in config types (operational defaults must come from development.yaml):\n{}",
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
