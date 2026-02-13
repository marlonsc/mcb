//! Tests for domain utility hashing helpers.

use mcb_domain::utils::compute_stable_id_hash;

#[test]
fn test_compute_stable_id_hash_is_deterministic() {
    let first = compute_stable_id_hash("session", "ses_1234567890");
    let second = compute_stable_id_hash("session", "ses_1234567890");

    assert_eq!(first, second);
}

#[test]
fn test_compute_stable_id_hash_changes_with_kind() {
    let session_hash = compute_stable_id_hash("session", "same-id");
    let project_hash = compute_stable_id_hash("project", "same-id");

    assert_ne!(session_hash, project_hash);
}

#[test]
fn test_compute_stable_id_hash_changes_with_value() {
    let first = compute_stable_id_hash("session", "ses-1");
    let second = compute_stable_id_hash("session", "ses-2");

    assert_ne!(first, second);
}

#[test]
fn test_compute_stable_id_hash_is_hex_encoded() {
    let hash = compute_stable_id_hash("session", "ses_abcdef");

    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|ch| ch.is_ascii_hexdigit()));
}
