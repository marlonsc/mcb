//! Snapshot Comparator Service - Compares snapshots to detect changes
//!
//! Single Responsibility: Compare two snapshots and identify differences.

use super::{CodebaseSnapshot, SnapshotChanges};

#[cfg(test)]
use super::FileSnapshot;
#[cfg(test)]
use std::collections::HashMap;

/// Service for comparing snapshots
pub struct SnapshotComparator;

impl Default for SnapshotComparator {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapshotComparator {
    /// Create a new comparator
    pub fn new() -> Self {
        Self
    }

    /// Compare two snapshots and return changes
    pub fn compare(
        &self,
        old_snapshot: &CodebaseSnapshot,
        new_snapshot: &CodebaseSnapshot,
    ) -> SnapshotChanges {
        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut removed = Vec::new();
        let mut unchanged = Vec::new();

        // Check each file in new snapshot
        for (path, new_file) in &new_snapshot.files {
            match old_snapshot.files.get(path) {
                Some(old_file) => {
                    // File exists in both - check if modified
                    if old_file.hash != new_file.hash {
                        modified.push(path.clone());
                    } else {
                        unchanged.push(path.clone());
                    }
                }
                None => {
                    // File is new
                    added.push(path.clone());
                }
            }
        }

        // Find removed files (in old but not in new)
        for path in old_snapshot.files.keys() {
            if !new_snapshot.files.contains_key(path) {
                removed.push(path.clone());
            }
        }

        SnapshotChanges {
            added,
            modified,
            removed,
            unchanged,
        }
    }

    /// Check if there are any changes
    pub fn has_changes(&self, changes: &SnapshotChanges) -> bool {
        !changes.added.is_empty() || !changes.modified.is_empty() || !changes.removed.is_empty()
    }

    /// Get total number of changes
    pub fn change_count(&self, changes: &SnapshotChanges) -> usize {
        changes.added.len() + changes.modified.len() + changes.removed.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_snapshot(files: Vec<(&str, &str)>) -> CodebaseSnapshot {
        let mut file_map = HashMap::new();
        for (path, hash) in files {
            file_map.insert(
                path.to_string(),
                FileSnapshot {
                    path: path.to_string(),
                    hash: hash.to_string(),
                    size: 100,
                    modified: 0,
                    extension: "rs".to_string(),
                },
            );
        }
        CodebaseSnapshot {
            root_path: "/test".to_string(),
            created_at: 0,
            files: file_map,
            file_count: 0,
            total_size: 0,
        }
    }

    #[test]
    fn test_no_changes() {
        let comparator = SnapshotComparator::new();
        let old = create_test_snapshot(vec![("a.rs", "hash1"), ("b.rs", "hash2")]);
        let new = create_test_snapshot(vec![("a.rs", "hash1"), ("b.rs", "hash2")]);

        let changes = comparator.compare(&old, &new);
        assert!(!comparator.has_changes(&changes));
        assert_eq!(changes.unchanged.len(), 2);
    }

    #[test]
    fn test_added_file() {
        let comparator = SnapshotComparator::new();
        let old = create_test_snapshot(vec![("a.rs", "hash1")]);
        let new = create_test_snapshot(vec![("a.rs", "hash1"), ("b.rs", "hash2")]);

        let changes = comparator.compare(&old, &new);
        assert!(comparator.has_changes(&changes));
        assert_eq!(changes.added, vec!["b.rs"]);
    }

    #[test]
    fn test_removed_file() {
        let comparator = SnapshotComparator::new();
        let old = create_test_snapshot(vec![("a.rs", "hash1"), ("b.rs", "hash2")]);
        let new = create_test_snapshot(vec![("a.rs", "hash1")]);

        let changes = comparator.compare(&old, &new);
        assert!(comparator.has_changes(&changes));
        assert_eq!(changes.removed, vec!["b.rs"]);
    }

    #[test]
    fn test_modified_file() {
        let comparator = SnapshotComparator::new();
        let old = create_test_snapshot(vec![("a.rs", "hash1")]);
        let new = create_test_snapshot(vec![("a.rs", "hash_changed")]);

        let changes = comparator.compare(&old, &new);
        assert!(comparator.has_changes(&changes));
        assert_eq!(changes.modified, vec!["a.rs"]);
    }

    #[test]
    fn test_change_count() {
        let comparator = SnapshotComparator::new();
        let old = create_test_snapshot(vec![("a.rs", "hash1"), ("b.rs", "hash2")]);
        let new = create_test_snapshot(vec![("a.rs", "hash_new"), ("c.rs", "hash3")]);

        let changes = comparator.compare(&old, &new);
        // a.rs modified, b.rs removed, c.rs added = 3 changes
        assert_eq!(comparator.change_count(&changes), 3);
    }
}
