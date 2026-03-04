use mcb_domain::utils::tests::fs_scan::rust_files_under;
use mcb_domain::utils::tests::utils::{TestResult, workspace_root};
use rstest::rstest;
use std::collections::BTreeMap;
use std::fs;

/// Maximum number of lines to look backwards from a `::new()` call to find
/// a `distributed_slice` attribute that proves the call lives inside a linkme
/// registry entry.
const REGISTRY_CONTEXT_WINDOW: usize = 15;

#[rstest]
#[test]
fn no_direct_concrete_di_shortcuts_outside_linkme_registries() -> TestResult {
    let root = workspace_root()?;

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

        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };

        let lines: Vec<&str> = content.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }

            for needle in forbidden {
                if line.contains(needle) {
                    // Check whether this call is inside a linkme registry entry
                    // by scanning up to REGISTRY_CONTEXT_WINDOW lines above for
                    // the `distributed_slice` attribute.
                    let start = idx.saturating_sub(REGISTRY_CONTEXT_WINDOW);
                    let in_registry = lines[start..idx]
                        .iter()
                        .any(|l| l.contains("distributed_slice"));

                    if !in_registry {
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
    }

    assert!(
        violations.is_empty(),
        "direct concrete DI shortcuts found outside linkme registries: {violations:#?}"
    );

    Ok(())
}
