use mcb_domain::utils::path::{
    path_to_utf8_string, strict_canonicalize, strict_strip_prefix, workspace_relative_path,
};
use std::path::{Path, PathBuf};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn workspace_relative_happy_path() -> TestResult {
    let root = Path::new("/home/user/project");
    let file = Path::new("/home/user/project/src/main.rs");
    assert_eq!(workspace_relative_path(file, root)?, "src/main.rs");
    Ok(())
}

#[test]
fn workspace_relative_nested() -> TestResult {
    let root = Path::new("/a/b");
    let file = Path::new("/a/b/c/d/e.rs");
    assert_eq!(workspace_relative_path(file, root)?, "c/d/e.rs");
    Ok(())
}

#[test]
fn workspace_relative_outside_root_returns_error() {
    let root = Path::new("/home/user/project");
    let file = Path::new("/other/place/file.rs");
    let result = workspace_relative_path(file, root);
    assert!(result.is_err(), "expected error for path outside root");
    if let Err(err) = result {
        assert!(
            err.to_string().contains("is not under root"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn strict_strip_prefix_same_path() -> TestResult {
    let root = Path::new("/a/b");
    let path = Path::new("/a/b");
    let result = strict_strip_prefix(path, root)?;
    assert_eq!(result, PathBuf::from(""));
    Ok(())
}

#[test]
fn strict_strip_prefix_errors_outside_root() {
    let root = Path::new("/a/b");
    let path = Path::new("/a/c/d");
    assert!(strict_strip_prefix(path, root).is_err());
}

#[test]
fn path_to_utf8_string_forward_slashes() -> TestResult {
    // On Unix the backslash is a valid filename char, but we still replace it
    let p = Path::new("src/main.rs");
    assert_eq!(path_to_utf8_string(p)?, "src/main.rs");
    Ok(())
}

#[test]
fn strict_canonicalize_nonexistent_path_returns_error() {
    let bad = Path::new("/this/path/definitely/does/not/exist/xyz123");
    let result = strict_canonicalize(bad);
    assert!(result.is_err(), "expected error for nonexistent path");
    if let Err(err) = result {
        assert!(
            err.to_string().contains("failed to canonicalize"),
            "unexpected error: {err}"
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn strict_canonicalize_real_path_succeeds() -> TestResult {
    // /tmp always exists on Linux
    let result = strict_canonicalize(Path::new("/tmp"))?;
    // Canonicalized /tmp might resolve symlinks, but should succeed
    assert!(result.exists());
    Ok(())
}
