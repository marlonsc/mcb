//! Tests for domain utility hashing helpers.

use mcb_domain::utils::compute_stable_id_hash;
use rstest::*;

#[test]
fn test_compute_stable_id_hash_is_deterministic() {
    let first = compute_stable_id_hash("session", "ses_1234567890");
    let second = compute_stable_id_hash("session", "ses_1234567890");

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
    let first = compute_stable_id_hash(kind_a, value_a);
    let second = compute_stable_id_hash(kind_b, value_b);
    assert_ne!(first, second);
}

#[test]
fn test_compute_stable_id_hash_is_hex_encoded() {
    let hash = compute_stable_id_hash("session", "ses_abcdef");

    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|ch| ch.is_ascii_hexdigit()));
}
