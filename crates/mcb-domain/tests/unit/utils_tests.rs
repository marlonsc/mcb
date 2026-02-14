//! Tests for domain utility hashing helpers.

use mcb_domain::utils::compute_stable_id_hash;
use rstest::rstest;

#[test]
fn test_compute_stable_id_hash_is_deterministic() {
    let first = compute_stable_id_hash("session", "ses_1234567890", None);
    let second = compute_stable_id_hash("session", "ses_1234567890", None);

    assert_eq!(first, second);
}

#[rstest]
#[case("session", "same-id", "project", "same-id")]
#[case("session", "ses-1", "session", "ses-2")]
fn compute_stable_id_hash_changes_with_input(
    #[case] kind_a: &str,
    #[case] value_a: &str,
    #[case] kind_b: &str,
    #[case] value_b: &str,
) {
    let first = compute_stable_id_hash(kind_a, value_a, None);
    let second = compute_stable_id_hash(kind_b, value_b, None);
    assert_ne!(first, second);
}

#[test]
fn test_compute_stable_id_hash_is_hex_encoded() {
    let hash = compute_stable_id_hash("session", "ses_abcdef", None);

    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|ch| ch.is_ascii_hexdigit()));
}

#[test]
fn test_compute_stable_id_hash_no_secret_pinned() {
    let hash = compute_stable_id_hash("session", "ses_1234567890", None);
    assert_eq!(
        hash,
        "14537b4d16fc0d90b25b87b4ee433d612992458d5a70715ef6c36ab8ea080fdd"
    );
}

#[test]
fn test_compute_stable_id_hash_with_secret_pinned() {
    let hash = compute_stable_id_hash("session", "ses_1234567890", Some("test-secret-key"));
    assert_eq!(
        hash,
        "1f3a9f61bca079ec9913c93e70e7c5282799b9e226ce8d9228092eba87a56112"
    );
}

#[test]
fn test_compute_stable_id_hash_empty_secret_uses_fallback() {
    let with_empty = compute_stable_id_hash("session", "ses_1234567890", Some(""));
    let without = compute_stable_id_hash("session", "ses_1234567890", None);
    assert_eq!(with_empty, without);
}

#[test]
fn test_compute_stable_id_hash_with_secret_is_deterministic() {
    let first = compute_stable_id_hash("session", "ses_1234567890", Some("my-secret"));
    let second = compute_stable_id_hash("session", "ses_1234567890", Some("my-secret"));
    assert_eq!(first, second);
}
