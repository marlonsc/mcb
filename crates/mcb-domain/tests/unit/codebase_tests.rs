//! Unit tests for Codebase entities
//!
//! Tests the CodebaseSnapshot and FileSnapshot entities, including
//! change tracking functionality.

use std::collections::HashMap;

use mcb_domain::entities::codebase::{CodebaseSnapshot, FileSnapshot, SnapshotChanges};
use rstest::rstest;

#[test]
fn test_file_snapshot_creation() {
    let file_snapshot = FileSnapshot {
        id: "file-001".to_string(),
        path: "src/lib.rs".to_string(),
        modified_at: 1641081600, // 2022-01-02 00:00:00 UTC
        size: 2048,
        hash: "def456".to_string(),
        language: "rust".to_string(),
    };

    assert_eq!(file_snapshot.path, "src/lib.rs");
    assert_eq!(file_snapshot.modified_at, 1641081600);
    assert_eq!(file_snapshot.size, 2048);
    assert_eq!(file_snapshot.hash, "def456");
    assert_eq!(file_snapshot.language, "rust");
}

#[test]
fn test_codebase_snapshot_creation() {
    let mut files = HashMap::new();
    files.insert(
        "src/main.rs".to_string(),
        FileSnapshot {
            id: "file-main".to_string(),
            path: "src/main.rs".to_string(),
            modified_at: 1640995200,
            size: 1024,
            hash: "abc123".to_string(),
            language: "rust".to_string(),
        },
    );

    let snapshot = CodebaseSnapshot {
        id: "snapshot-001".to_string(),
        created_at: 1640995200,
        collection: "my-project".to_string(),
        files: files.clone(),
        total_files: 1,
        total_size: 1024,
    };

    assert_eq!(snapshot.id, "snapshot-001");
    assert_eq!(snapshot.created_at, 1640995200);
    assert_eq!(snapshot.collection, "my-project");
    assert_eq!(snapshot.total_files, 1);
    assert_eq!(snapshot.total_size, 1024);
    assert_eq!(snapshot.files.len(), 1);
    assert!(snapshot.files.contains_key("src/main.rs"));
}

#[test]
fn test_codebase_snapshot_multiple_files() {
    let mut files = HashMap::new();

    files.insert(
        "src/main.rs".to_string(),
        FileSnapshot {
            id: "file-main-2".to_string(),
            path: "src/main.rs".to_string(),
            modified_at: 1640995200,
            size: 1024,
            hash: "abc123".to_string(),
            language: "rust".to_string(),
        },
    );

    files.insert(
        "src/lib.rs".to_string(),
        FileSnapshot {
            id: "file-lib".to_string(),
            path: "src/lib.rs".to_string(),
            modified_at: 1641081600,
            size: 2048,
            hash: "def456".to_string(),
            language: "rust".to_string(),
        },
    );

    files.insert(
        "Cargo.toml".to_string(),
        FileSnapshot {
            id: "file-cargo".to_string(),
            path: "Cargo.toml".to_string(),
            modified_at: 1640995200,
            size: 512,
            hash: "toml123".to_string(),
            language: "toml".to_string(),
        },
    );

    let snapshot = CodebaseSnapshot {
        id: "multi-file-snapshot".to_string(),
        created_at: 1641081600,
        collection: "test-project".to_string(),
        files: files.clone(),
        total_files: 3,
        total_size: 3584, // 1024 + 2048 + 512
    };

    assert_eq!(snapshot.total_files, 3);
    assert_eq!(snapshot.total_size, 3584);
    assert_eq!(snapshot.files.len(), 3);
    assert!(snapshot.files.contains_key("src/main.rs"));
    assert!(snapshot.files.contains_key("src/lib.rs"));
    assert!(snapshot.files.contains_key("Cargo.toml"));
}

#[rstest]
#[case(vec![], vec![], vec![], false, 0)]
#[case(vec!["new_file.rs", "another.rs"], vec![], vec![], true, 2)]
#[case(vec!["new.rs"], vec!["changed.rs", "updated.rs"], vec!["deleted.rs"], true, 4)]
#[case(vec![], vec!["modified1.rs", "modified2.rs", "modified3.rs"], vec![], true, 3)]
#[case(vec![], vec![], vec!["gone1.rs", "gone2.rs"], true, 2)]
#[test]
fn test_snapshot_changes_variants(
    #[case] added: Vec<&str>,
    #[case] modified: Vec<&str>,
    #[case] removed: Vec<&str>,
    #[case] expected_has_changes: bool,
    #[case] expected_total: usize,
) {
    let changes = SnapshotChanges {
        added: added.into_iter().map(str::to_string).collect(),
        modified: modified.into_iter().map(str::to_string).collect(),
        removed: removed.into_iter().map(str::to_string).collect(),
    };

    assert_eq!(changes.has_changes(), expected_has_changes);
    assert_eq!(changes.total_changes(), expected_total);
}
