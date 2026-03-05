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

    // Filter out fixture directories so we only scan production/intentional DI sites.
    files.retain(|path| {
        !path
            .components()
            .any(|c| c.as_os_str().to_string_lossy() == "fixtures")
    });
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

        let mut in_block_comment = false;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Track block comments
            if in_block_comment {
                if trimmed.contains("*/") {
                    in_block_comment = false;
                }
                continue;
            }
            if trimmed.starts_with("/*") {
                in_block_comment = true;
                if trimmed.contains("*/") {
                    in_block_comment = false;
                }
                continue;
            }
            if trimmed.starts_with("//") {
                continue;
            }

            for needle in forbidden {
                if !line.contains(needle) {
                    continue;
                }

                // Skip matches that appear inside string literals.
                // Heuristic: if a quote appears before the needle on the same
                // line, the needle is likely inside a string.
                if let Some(pos) = line.find(needle) {
                    let before = &line[..pos];
                    if before.contains('"') {
                        continue;
                    }
                }

                // Check whether this call is inside a linkme registry entry
                // by scanning up to REGISTRY_CONTEXT_WINDOW lines above for
                // specific registry-related attributes/macros.
                let start = idx.saturating_sub(REGISTRY_CONTEXT_WINDOW);
                let in_registry = lines[start..idx].iter().any(|l| {
                    l.contains("#[linkme::distributed_slice]")
                        || l.contains("linkme::distributed_slice")
                        || l.contains("register_tool!")
                        || (l.contains("register_") && l.contains("_provider!"))
                });

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

    assert!(
        violations.is_empty(),
        "direct concrete DI shortcuts found outside linkme registries: {violations:#?}"
    );

    Ok(())
}
