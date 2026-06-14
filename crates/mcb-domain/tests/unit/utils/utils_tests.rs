use mcb_domain::utils::tests::utils::TestResult;
use mcb_utils::utils::id;
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
fn correlate_id_returns_valid_uuid_format() -> TestResult {
    let result = id::correlate_id("session", "ses_abcdef");
    let parsed = uuid::Uuid::parse_str(&result)?;
    assert_eq!(
        parsed.get_version_num(),
        5,
        "correlate_id must return a UUID v5 string"
    );
    Ok(())
}

#[rstest]
fn correlate_id_pinned_value() {
    let result = id::correlate_id("session", "ses_1234567890");
    assert_eq!(
        result,
        id::deterministic("session", "ses_1234567890").to_string()
    );
}

#[rstest]
fn deterministic_uuid_is_v5() {
    let uuid = id::deterministic("session", "ses_1234567890");
    assert_eq!(uuid.get_version_num(), 5);
}

#[rstest]
fn generate_returns_v4() {
    let uuid = id::generate();
    assert_eq!(uuid.get_version_num(), 4);
}

#[rstest]
fn generate_is_unique() {
    let a = id::generate();
    let b = id::generate();
    assert_ne!(a, b);
}
