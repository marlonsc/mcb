//! Unit tests for Codebase entities
//!
//! Tests the CodebaseSnapshot and FileSnapshot entities, including
//! change tracking functionality.

use rstest::rstest;
use std::collections::HashMap;

use mcb_domain::entities::codebase::{CodebaseSnapshot, FileSnapshot, SnapshotChanges};

#[rstest]
#[case("src/lib.rs", 1641081600, 2048, "def456", "rust")]
fn file_snapshot_creation(
    #[case] path: &str,
    #[case] modified_at: i64,
    #[case] size: u64,
    #[case] hash: &str,
    #[case] language: &str,
) {
    let file_snapshot = FileSnapshot {
        id: "file-001".to_string(),
        path: path.to_string(),
        modified_at,
        size,
        hash: hash.to_string(),
        language: language.to_string(),
    };

    assert_eq!(file_snapshot.path, path);
    assert_eq!(file_snapshot.modified_at, modified_at);
    assert_eq!(file_snapshot.size, size);
    assert_eq!(file_snapshot.hash, hash);
    assert_eq!(file_snapshot.language, language);
}

#[rstest]
#[case(false)]
#[case(true)]
fn codebase_snapshot_creation(#[case] multiple_files: bool) {
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

    if multiple_files {
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
    }

    let snapshot = CodebaseSnapshot {
        id: if multiple_files {
            "multi-file-snapshot".to_string()
        } else {
            "snapshot-001".to_string()
        },
        created_at: if multiple_files {
            1641081600
        } else {
            1640995200
        },
        collection: if multiple_files {
            "test-project".to_string()
        } else {
            "my-project".to_string()
        },
        files: files.clone(),
        total_files: files.len(),
        total_size: files.values().map(|f| f.size).sum(),
    };

    assert_eq!(snapshot.total_files, if multiple_files { 3 } else { 1 });
    assert_eq!(
        snapshot.total_size,
        if multiple_files { 3584 } else { 1024 }
    );
    assert_eq!(snapshot.files.len(), if multiple_files { 3 } else { 1 });
    assert!(snapshot.files.contains_key("src/main.rs"));
    if multiple_files {
        assert!(snapshot.files.contains_key("src/lib.rs"));
        assert!(snapshot.files.contains_key("Cargo.toml"));
    }
}

#[rstest]
#[case(vec![], vec![], vec![], false, 0)]
#[case(vec!["new_file.rs", "another.rs"], vec![], vec![], true, 2)]
#[case(vec!["new.rs"], vec!["changed.rs", "updated.rs"], vec!["deleted.rs"], true, 4)]
#[case(vec![], vec!["modified1.rs", "modified2.rs", "modified3.rs"], vec![], true, 3)]
#[case(vec![], vec![], vec!["gone1.rs", "gone2.rs"], true, 2)]
fn snapshot_changes_variants(
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
