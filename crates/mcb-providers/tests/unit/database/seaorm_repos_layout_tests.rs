use std::path::Path;

const SEAORM_REPOS_COMMON_MAX_LINES: usize = 500;

/// Guards the `SeaORM` shared macro module split required by the WS-HARD MVI lane.
#[test]
fn seaorm_common_module_stays_below_mvi_split_limit() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/database/seaorm/repos/common.rs");
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => panic!("failed to read {}: {error}", path.display()),
    };
    let line_count = source.lines().count();

    assert!(
        line_count < SEAORM_REPOS_COMMON_MAX_LINES,
        "{} has {line_count} lines, expected fewer than {SEAORM_REPOS_COMMON_MAX_LINES}",
        path.display(),
    );
}
