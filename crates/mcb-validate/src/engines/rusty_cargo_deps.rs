//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Cargo dependency scanning for the `cargo_dependencies` rusty-rule type.
//!
//! Walks a workspace tree looking for a `Cargo.toml` whose `[dependencies]`
//! declare a key matching a forbidden prefix, used by
//! [`super::rusty_rules_engine`].

/// Returns true when any `Cargo.toml` under `workspace_root` declares a
/// dependency whose name starts with `pattern_prefix`.
pub(crate) fn workspace_has_forbidden_cargo_dependency(
    workspace_root: &std::path::Path,
    pattern_prefix: &str,
) -> bool {
    let mut stack = vec![workspace_root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if cargo_toml_matches(&path, pattern_prefix) {
                return true;
            }
        }
    }

    false
}

/// Returns true when `path` is a `Cargo.toml` whose dependencies match `pattern_prefix`.
fn cargo_toml_matches(path: &std::path::Path, pattern_prefix: &str) -> bool {
    if path.file_name().and_then(std::ffi::OsStr::to_str) != Some("Cargo.toml") {
        return false;
    }

    std::fs::read_to_string(path)
        .is_ok_and(|content| dependency_matches(content.as_ref(), pattern_prefix))
}

/// Returns true when `content` (a `Cargo.toml`) declares a dependency whose
/// name starts with `pattern_prefix`, via TOML parse or line scan.
pub(crate) fn dependency_matches(content: &str, pattern_prefix: &str) -> bool {
    content
        .parse::<toml::Value>()
        .ok()
        .is_some_and(|toml_value| toml_dependencies_match(&toml_value, pattern_prefix))
        || dependencies_match_by_line(content, pattern_prefix)
}

fn toml_dependencies_match(toml_value: &toml::Value, pattern_prefix: &str) -> bool {
    let Some(dependencies) = toml_value.get("dependencies") else {
        return false;
    };
    let Some(deps_table) = dependencies.as_table() else {
        return false;
    };

    deps_table
        .keys()
        .any(|dep_name| dep_name.starts_with(pattern_prefix))
}

fn dependencies_match_by_line(content: &str, pattern_prefix: &str) -> bool {
    let mut in_dependencies = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_dependencies = trimmed == "[dependencies]";
            continue;
        }

        if in_dependencies && dependency_line_matches(trimmed, pattern_prefix) {
            return true;
        }
    }

    false
}

/// Returns true when a `[dependencies]` line declares a key starting with `pattern_prefix`.
fn dependency_line_matches(trimmed: &str, pattern_prefix: &str) -> bool {
    let Some((key, _)) = trimmed.split_once('=') else {
        return false;
    };
    let dep_name = key.trim().trim_matches('"').trim_matches('\'');
    dep_name.starts_with(pattern_prefix)
}
