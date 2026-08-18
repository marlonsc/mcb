use mcb_domain::utils::tests::utils::{TestResult, workspace_root};
use rstest::rstest;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
struct AstPosition {
    line: usize,
    column: usize,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
struct AstRange {
    start: AstPosition,
    end: AstPosition,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct AstMatch {
    file: String,
    range: AstRange,
}

impl AstMatch {
    fn contains(&self, candidate: &Self) -> bool {
        self.file == candidate.file
            && self.range.start <= candidate.range.start
            && candidate.range.end <= self.range.end
    }
}

fn parse_ast_matches(output: &[u8]) -> TestResult<Vec<AstMatch>> {
    String::from_utf8(output.to_vec())?
        .lines()
        .map(|line| serde_json::from_str(line).map_err(Into::into))
        .collect()
}

fn semantic_constructor_matches(path: &Path, constructor: &str) -> TestResult<Vec<AstMatch>> {
    let pattern = format!("{constructor}($$$ARGS)");
    let output = Command::new("ast-grep")
        .args([
            "run",
            "--pattern",
            &pattern,
            "--lang",
            "rust",
            "--json=stream",
        ])
        .arg(path)
        .output()?;
    // ast-grep run exits 1 with empty stderr when zero matches exist; any
    // other non-zero exit or stderr output is a broken scan and must fail.
    let zero_matches =
        output.status.code() == Some(1) && output.stderr.is_empty() && output.stdout.is_empty();
    assert!(
        output.status.success() || zero_matches,
        "ast-grep failed for {}: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );

    parse_ast_matches(&output.stdout)
}

/// Registration containment applies to parsed code only: macro bodies are
/// token trees, so constructor calls inside `register_*!` invocations never
/// produce constructor matches and are implicitly authorized.
fn semantic_registration_matches(path: &Path) -> TestResult<Vec<AstMatch>> {
    const REGISTRATION_RULES: &str = "
id: distributed-slice-registration
language: Rust
rule:
  kind: static_item
  follows:
    kind: attribute_item
    regex: 'linkme::distributed_slice'
";
    let output = Command::new("ast-grep")
        .args([
            "scan",
            "--inline-rules",
            REGISTRATION_RULES,
            "--json=stream",
        ])
        .arg(path)
        .output()?;
    assert!(
        output.status.success(),
        "ast-grep registration scan failed for {}: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    parse_ast_matches(&output.stdout)
}

#[test]
fn semantic_constructor_match_detects_arguments() -> TestResult {
    let fixture_dir = tempfile::tempdir()?;
    let fixture = fixture_dir.path().join("constructor.rs");
    fs::write(
        &fixture,
        "fn build(dep: Dep) { HighlightServiceImpl::new(dep); }",
    )?;

    let matches = semantic_constructor_matches(&fixture, "HighlightServiceImpl::new")?;

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].file, fixture.display().to_string());
    assert_eq!(matches[0].range.start.line, 0);
    Ok(())
}

#[test]
fn registration_containment_authorizes_slice_entries_and_flags_bypass() -> TestResult {
    let fixture_dir = tempfile::tempdir()?;
    let fixture = fixture_dir.path().join("registrations.rs");
    fs::write(
        &fixture,
        r#"
#[linkme::distributed_slice(REGISTRY)]
static ENTRY: Factory = Factory { build: |dep| HighlightServiceImpl::new(dep) };

mcb_domain::register_service!("highlight", || {
    HighlightServiceImpl::new(dep)
});

fn bypass(dep: Dep) {
    HighlightServiceImpl::new(dep);
}
"#,
    )?;

    let constructors = semantic_constructor_matches(&fixture, "HighlightServiceImpl::new")?;
    let registrations = semantic_registration_matches(&fixture)?;

    assert_eq!(
        constructors.len(),
        2,
        "macro token-tree constructors must stay invisible to expression patterns"
    );
    assert!(
        registrations
            .iter()
            .any(|registration| registration.contains(&constructors[0]))
    );
    assert!(
        !registrations
            .iter()
            .any(|registration| registration.contains(&constructors[1]))
    );
    Ok(())
}

#[rstest]
fn no_direct_concrete_di_shortcuts_outside_linkme_registries() -> TestResult {
    let root = workspace_root()?;

    let forbidden = [
        "DefaultIndexingOperations::new",
        "DefaultValidationOperations::new",
        "BroadcastEventBus::new",
        "GitProvider::new",
        "HighlightServiceImpl::new",
        "InfraValidationService::new",
    ];

    let crates_root = root.join("crates");
    let mut violations: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for constructor in forbidden {
        for constructor_match in semantic_constructor_matches(&crates_root, constructor)? {
            let file = Path::new(&constructor_match.file);
            if file
                .components()
                .any(|component| component.as_os_str().to_string_lossy() == "fixtures")
                || file
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name == "di_strict_enforcement_tests.rs")
            {
                continue;
            }

            let registrations = semantic_registration_matches(file)?;
            if !registrations
                .iter()
                .any(|registration| registration.contains(&constructor_match))
            {
                let rel = file
                    .strip_prefix(&root)
                    .unwrap_or(file)
                    .display()
                    .to_string();
                violations.entry(rel).or_default().push(format!(
                    "{}:{}",
                    constructor_match.range.start.line + 1,
                    constructor
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "direct concrete DI shortcuts found outside linkme registries: {violations:#?}"
    );

    Ok(())
}
