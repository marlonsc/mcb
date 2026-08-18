//! Tests for naming validation.

use std::{fs, io};

use mcb_domain::utils::tests::assertions::assert_no_violations;
use mcb_validate::NamingValidator;
use rstest::rstest;
use tempfile::TempDir;

use crate::utils::test_constants::TEST_CRATE;
use crate::utils::with_inline_crate;

fn write_crate_source(
    root: &TempDir,
    crate_name: &str,
    relative_path: &str,
    content: &str,
) -> io::Result<()> {
    let workspace_manifest = root.path().join("Cargo.toml");
    fs::write(
        &workspace_manifest,
        "[workspace]\nmembers = [\"crates/*\"]\n",
    )?;

    let crate_root = root.path().join("crates").join(crate_name);
    let source_path = crate_root.join("src").join(relative_path);
    let source_dir = source_path
        .parent()
        .ok_or_else(|| io::Error::other("source path has no parent"))?;
    fs::create_dir_all(source_dir)?;
    fs::write(source_path, content)?;
    fs::write(
        crate_root.join("Cargo.toml"),
        format!("[package]\nname = \"{crate_name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
    )?;
    Ok(())
}

#[rstest]
fn lifetime_static_path_type_is_not_static_declaration() -> mcb_validate::Result<()> {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        "
pub fn table() -> Option<&'static toml::value::Table> {
    None
}
",
    );
    let validator = NamingValidator::new(&root);
    let violations = validator.validate_all()?;

    assert_no_violations(
        &violations,
        "Lifetime 'static followed by a path type is not a static declaration",
    );
    Ok(())
}

#[rstest]
fn repository_entity_file_is_not_repository_port() -> io::Result<()> {
    let temp = TempDir::new()?;
    write_crate_source(
        &temp,
        "mcb-domain",
        "entities/repository.rs",
        "pub struct Repository;\n",
    )?;

    let validator = NamingValidator::new(temp.path());
    let violations = validator
        .validate_all()
        .map_err(|e| io::Error::other(e.to_string()))?;

    assert_no_violations(
        &violations,
        "Entity files named repository.rs are domain entities, not repository ports",
    );
    Ok(())
}

#[rstest]
fn handler_macro_file_is_not_http_handler() -> io::Result<()> {
    let temp = TempDir::new()?;
    write_crate_source(
        &temp,
        "mcb-server",
        "macros/handlers.rs",
        "macro_rules! tool_handler { () => {} }\n",
    )?;

    let validator = NamingValidator::new(temp.path());
    let violations = validator
        .validate_all()
        .map_err(|e| io::Error::other(e.to_string()))?;

    assert_no_violations(
        &violations,
        "Macro helper files named handlers.rs are not HTTP or MCP handler modules",
    );
    Ok(())
}
