use mcb_domain::value_objects::CollectionId;
use mcb_providers::constants::{
    MILVUS_COLLECTION_NAME_PATTERN, VECTOR_FIELD_FILE_PATH, VECTOR_FIELD_ID,
    VECTOR_FIELD_START_LINE,
};
use mcb_providers::vector_store::milvus::browser::convert_query_results;
use mcb_providers::vector_store::milvus::schema::{extract_long_field, extract_string_field};
use mcb_providers::vector_store::milvus::to_milvus_name;
use milvus::data::FieldColumn;
use milvus::proto::schema::DataType;
use milvus::value::ValueVec;
use rstest::{fixture, rstest};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

#[fixture]
fn collection_id() -> CollectionId {
    CollectionId::from_name("test-collection")
}

#[fixture]
fn milvus_name(collection_id: CollectionId) -> String {
    to_milvus_name(&collection_id)
}

// ---------------------------------------------------------------------------
// to_milvus_name – validated through a single parametrized test
// ---------------------------------------------------------------------------

#[rstest]
#[test]
fn test_to_milvus_name_starts_with_letter(milvus_name: String) {
    assert!(
        milvus_name.starts_with("mcb_"),
        "name must start with mcb_ prefix: {milvus_name}"
    );
}

#[rstest]
#[test]
fn test_to_milvus_name_no_hyphens(milvus_name: String) {
    assert!(
        !milvus_name.contains('-'),
        "name must not contain hyphens: {milvus_name}"
    );
}

#[rstest]
#[test]
fn test_to_milvus_name_valid_pattern(milvus_name: String) {
    let pattern = regex::Regex::new(MILVUS_COLLECTION_NAME_PATTERN).unwrap();
    assert!(
        pattern.is_match(&milvus_name),
        "name must match Milvus pattern: {milvus_name}"
    );
}

#[rstest]
#[test]
fn test_to_milvus_name_under_255_chars(milvus_name: String) {
    assert!(
        milvus_name.len() <= 255,
        "name must be under 255 chars: {milvus_name}"
    );
}

// ---------------------------------------------------------------------------
// Helpers for FieldColumn construction
// ---------------------------------------------------------------------------

#[fixture]
fn string_column() -> impl Fn(&str, Vec<String>) -> FieldColumn {
    |name: &str, values: Vec<String>| FieldColumn {
        name: name.to_owned(),
        dtype: DataType::VarChar,
        value: ValueVec::String(values),
        dim: 1,
        max_length: 256,
        is_dynamic: false,
    }
}

#[fixture]
fn long_column() -> impl Fn(&str, Vec<i64>) -> FieldColumn {
    |name: &str, values: Vec<i64>| FieldColumn {
        name: name.to_owned(),
        dtype: DataType::Int64,
        value: ValueVec::Long(values),
        dim: 1,
        max_length: 0,
        is_dynamic: false,
    }
}

// ---------------------------------------------------------------------------
// extract_string_field / extract_long_field
// ---------------------------------------------------------------------------

#[rstest]
#[test]
fn test_extract_string_field_missing_column_returns_error() {
    let fields: Vec<FieldColumn> = vec![];
    let result = extract_string_field(&fields, "missing", 0);
    let err = result.expect_err("extract_string_field should fail for missing column");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("missing string field"),
        "Expected error about missing field, got: {err_msg}"
    );
}

#[rstest]
#[test]
fn test_extract_string_field_out_of_bounds_returns_error(
    string_column: impl Fn(&str, Vec<String>) -> FieldColumn,
) {
    let fields = vec![string_column(
        VECTOR_FIELD_FILE_PATH,
        vec!["a.rs".to_owned()],
    )];
    let result = extract_string_field(&fields, VECTOR_FIELD_FILE_PATH, 99);
    let err = result.expect_err("extract_string_field should fail for out-of-bounds index");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("missing string field"),
        "Expected error about missing field, got: {err_msg}"
    );
}

#[rstest]
#[test]
fn test_extract_string_field_valid_returns_ok(
    string_column: impl Fn(&str, Vec<String>) -> FieldColumn,
) {
    let fields = vec![string_column(
        VECTOR_FIELD_FILE_PATH,
        vec!["src/main.rs".to_owned()],
    )];
    let result = extract_string_field(&fields, VECTOR_FIELD_FILE_PATH, 0);
    assert_eq!(result.unwrap(), "src/main.rs");
}

#[rstest]
#[test]
fn test_extract_long_field_missing_column_returns_error() {
    let fields: Vec<FieldColumn> = vec![];
    let result = extract_long_field(&fields, "missing", 0);
    let err = result.expect_err("extract_long_field should fail for missing column");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("missing long field"),
        "Expected error about missing field, got: {err_msg}"
    );
}

#[rstest]
#[test]
fn test_extract_long_field_valid_returns_ok(long_column: impl Fn(&str, Vec<i64>) -> FieldColumn) {
    let fields = vec![long_column(VECTOR_FIELD_START_LINE, vec![42])];
    let result = extract_long_field(&fields, VECTOR_FIELD_START_LINE, 0);
    assert_eq!(result.unwrap(), 42);
}

#[rstest]
#[test]
fn test_convert_query_results_missing_fields_returns_error(
    string_column: impl Fn(&str, Vec<String>) -> FieldColumn,
) {
    let fields: Vec<FieldColumn> = vec![string_column(VECTOR_FIELD_ID, vec!["1".to_owned()])];
    let result = convert_query_results(&fields, None);
    let err =
        result.expect_err("convert_query_results should fail when required fields are missing");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("missing"),
        "Expected error about missing field, got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// Error propagation — parametrized with #[case]
// ---------------------------------------------------------------------------

#[rstest]
#[case(
    "test-col",
    "Collection '{}' not found when listing file paths",
    &["not found", "listing file paths"]
)]
#[case(
    "my-collection",
    "Failed to query file paths in collection '{}': connection refused",
    &["Failed to query file paths", "connection refused"]
)]
#[case(
    "chunks-col",
    "Failed to query chunks by file in collection '{}': timeout",
    &["Failed to query chunks by file", "timeout"]
)]
#[case(
    "missing-col",
    "Collection '{}' not found when querying chunks by file",
    &["not found", "chunks by file"]
)]
fn test_error_message_contains_expected_substrings(
    #[case] collection_name: &str,
    #[case] msg_template: &str,
    #[case] expected_substrings: &[&str],
) {
    let collection = CollectionId::from_name(collection_name);
    let msg = msg_template.replace("{}", &collection.to_string());
    let err = mcb_domain::error::Error::vector_db(msg);
    let err_str = err.to_string();
    for substring in expected_substrings {
        assert!(
            err_str.contains(substring),
            "Error should contain '{substring}': {err_str}"
        );
    }
}

// ---------------------------------------------------------------------------
// DEFAULT_OUTPUT_FIELDS
// ---------------------------------------------------------------------------

#[rstest]
#[case(VECTOR_FIELD_ID)]
#[case(VECTOR_FIELD_FILE_PATH)]
#[case(VECTOR_FIELD_START_LINE)]
#[case(mcb_providers::constants::VECTOR_FIELD_CONTENT)]
fn test_default_output_fields_contains_field(#[case] field: &str) {
    use mcb_providers::vector_store::milvus::DEFAULT_OUTPUT_FIELDS;
    assert!(
        DEFAULT_OUTPUT_FIELDS.contains(&field),
        "DEFAULT_OUTPUT_FIELDS must contain '{field}' for extraction to work"
    );
}
