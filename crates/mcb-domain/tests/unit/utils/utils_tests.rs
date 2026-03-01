use mcb_domain::utils::id;
use rstest::rstest;

#[rstest]
fn correlate_id_is_deterministic() {
    let first = id::correlate_id("session", "ses_1234567890");
    let second = id::correlate_id("session", "ses_1234567890");
    assert_eq!(first, second);
}

#[rstest]
#[case("session", "same-id", "project", "same-id")]
#[case("session", "ses-1", "session", "ses-2")]
fn correlate_id_changes_with_input(
    #[case] kind_a: &str,
    #[case] value_a: &str,
    #[case] kind_b: &str,
    #[case] value_b: &str,
) {
    let first = id::correlate_id(kind_a, value_a);
    let second = id::correlate_id(kind_b, value_b);
    assert_ne!(first, second);
}

#[rstest]
#[test]
fn correlate_id_returns_valid_uuid_format() {
    let result = id::correlate_id("session", "ses_abcdef");
    let parsed = uuid::Uuid::parse_str(&result);
    assert!(
        parsed.is_ok(),
        "correlate_id must return a valid UUID string"
    );
    assert_eq!(parsed.unwrap().get_version_num(), 5);
}

#[rstest]
#[test]
fn correlate_id_pinned_value() {
    let result = id::correlate_id("session", "ses_1234567890");
    assert_eq!(
        result,
        id::deterministic("session", "ses_1234567890").to_string()
    );
}

#[rstest]
#[test]
fn deterministic_uuid_is_v5() {
    let uuid = id::deterministic("session", "ses_1234567890");
    assert_eq!(uuid.get_version_num(), 5);
}

#[rstest]
#[test]
fn generate_returns_v4() {
    let uuid = id::generate();
    assert_eq!(uuid.get_version_num(), 4);
}

#[rstest]
#[test]
fn generate_is_unique() {
    let a = id::generate();
    let b = id::generate();
    assert_ne!(a, b);
}
