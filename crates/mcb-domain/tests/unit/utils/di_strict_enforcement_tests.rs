use rstest::rstest;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf()
}

fn rust_files_under(path: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                rust_files_under(&p, out);
            } else if p.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(p);
            }
        }
    }
}

#[rstest]
#[test]
fn no_direct_concrete_di_shortcuts_outside_linkme_registries() {
    let root = workspace_root();

    let allowed_paths: Vec<PathBuf> = vec![
        root.join("crates/mcb-infrastructure/src/infrastructure/admin.rs"),
        root.join("crates/mcb-infrastructure/src/events/broadcast.rs"),
        root.join("crates/mcb-providers/src/vcs/git.rs"),
        root.join("crates/mcb-infrastructure/src/services/highlight_service.rs"),
        root.join("crates/mcb-infrastructure/src/validation/service.rs"),
    ];

    let forbidden = [
        "DefaultIndexingOperations::new()",
        "DefaultValidationOperations::new()",
        "BroadcastEventBus::new()",
        "GitProvider::new()",
        "HighlightServiceImpl::new()",
        "InfraValidationService::new()",
    ];

    let mut files = Vec::new();
    rust_files_under(&root.join("crates"), &mut files);

    let mut violations: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for file in files {
        if file
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n == "di_strict_enforcement_tests.rs")
        {
            continue;
        }

        if allowed_paths.iter().any(|allowed| allowed == &file) {
            continue;
        }

        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };

        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }

            for needle in forbidden {
                if line.contains(needle) {
                    let rel = file
                        .strip_prefix(&root)
                        .unwrap_or(&file)
                        .display()
                        .to_string();
                    violations
                        .entry(rel)
                        .or_default()
                        .push(format!("{}:{}", idx + 1, needle));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "direct concrete DI shortcuts found outside linkme registries: {violations:#?}"
    );
}
