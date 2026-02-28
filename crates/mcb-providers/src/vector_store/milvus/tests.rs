use super::*;
use mcb_domain::value_objects::{CollectionId, Embedding};
use milvus::data::FieldColumn;
use milvus::proto::schema::DataType;
use milvus::value::ValueVec;
use std::collections::HashMap;

#[test]
fn test_to_milvus_name_starts_with_letter() {
    let id = CollectionId::from_name("test-collection");
    let name = to_milvus_name(&id);
    assert!(
        name.starts_with("mcb_"),
        "name must start with mcb_ prefix: {name}"
    );
}

#[test]
fn test_to_milvus_name_no_hyphens() {
    let id = CollectionId::from_name("test-collection");
    let name = to_milvus_name(&id);
    assert!(!name.contains('-'), "name must not contain hyphens: {name}");
}

#[test]
fn test_to_milvus_name_valid_pattern() {
    let id = CollectionId::from_name("test-collection");
    let name = to_milvus_name(&id);
    let pattern = regex::Regex::new(crate::constants::MILVUS_COLLECTION_NAME_PATTERN).unwrap();
    assert!(
        pattern.is_match(&name),
        "name must match Milvus pattern: {name}"
    );
}

#[test]
fn test_to_milvus_name_under_255_chars() {
    let id = CollectionId::from_name("test-collection");
    let name = to_milvus_name(&id);
    assert!(name.len() <= 255, "name must be under 255 chars: {name}");
}

// --- Error propagation tests for malformed Milvus responses ---

/// Helper: build a `FieldColumn` with string values
fn make_string_column(name: &str, values: Vec<String>) -> FieldColumn {
    FieldColumn {
        name: name.to_owned(),
        dtype: DataType::VarChar,
        value: ValueVec::String(values),
        dim: 1,
        max_length: 256,
        is_dynamic: false,
    }
}

/// Helper: build a `FieldColumn` with long values
fn make_long_column(name: &str, values: Vec<i64>) -> FieldColumn {
    FieldColumn {
        name: name.to_owned(),
        dtype: DataType::Int64,
        value: ValueVec::Long(values),
        dim: 1,
        max_length: 0,
        is_dynamic: false,
    }
}

#[test]
fn test_extract_string_field_missing_column_returns_error() {
    let fields: Vec<FieldColumn> = vec![];
    let result = schema::extract_string_field(&fields, "missing", 0);
    let err = result.expect_err("extract_string_field should fail for missing column");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("missing string field"),
        "Expected error about missing field, got: {err_msg}"
    );
}

#[test]
fn test_extract_string_field_out_of_bounds_returns_error() {
    let fields = vec![make_string_column(
        crate::constants::VECTOR_FIELD_FILE_PATH,
        vec!["a.rs".to_owned()],
    )];
    let result =
        schema::extract_string_field(&fields, crate::constants::VECTOR_FIELD_FILE_PATH, 99);
    let err = result.expect_err("extract_string_field should fail for out-of-bounds index");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("missing string field"),
        "Expected error about missing field, got: {err_msg}"
    );
}

#[test]
fn test_extract_string_field_valid_returns_ok() {
    let fields = vec![make_string_column(
        crate::constants::VECTOR_FIELD_FILE_PATH,
        vec!["src/main.rs".to_owned()],
    )];
    let result = schema::extract_string_field(&fields, crate::constants::VECTOR_FIELD_FILE_PATH, 0);
    assert_eq!(result.unwrap(), "src/main.rs");
}

#[test]
fn test_extract_long_field_missing_column_returns_error() {
    let fields: Vec<FieldColumn> = vec![];
    let result = schema::extract_long_field(&fields, "missing", 0);
    let err = result.expect_err("extract_long_field should fail for missing column");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("missing long field"),
        "Expected error about missing field, got: {err_msg}"
    );
}

#[test]
fn test_extract_long_field_valid_returns_ok() {
    let fields = vec![make_long_column(
        crate::constants::VECTOR_FIELD_START_LINE,
        vec![42],
    )];
    let result = schema::extract_long_field(&fields, crate::constants::VECTOR_FIELD_START_LINE, 0);
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_convert_query_results_missing_fields_returns_error() {
    // Empty field columns — extract_string_field should fail
    let fields: Vec<FieldColumn> = vec![make_string_column(
        crate::constants::VECTOR_FIELD_ID,
        vec!["1".to_owned()],
    )];
    let result = browser::convert_query_results(&fields, None);
    let err =
        result.expect_err("convert_query_results should fail when required fields are missing");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("missing"),
        "Expected error about missing field, got: {err_msg}"
    );
}

// --- Error propagation tests for query/search failure paths ---

#[test]
fn test_error_collection_not_found_listing_file_paths() {
    let collection = CollectionId::from_name("test-col");
    let err = mcb_domain::error::Error::vector_db(format!(
        "Collection '{collection}' not found when listing file paths"
    ));
    let msg = err.to_string();
    assert!(
        msg.contains("not found") && msg.contains("listing file paths"),
        "Error should contain 'not found' and 'listing file paths': {msg}"
    );
}

#[test]
fn test_error_query_file_paths_propagates_cause() {
    let collection = CollectionId::from_name("my-collection");
    let original_err = "connection refused";
    let msg = format!("Failed to query file paths in collection '{collection}': {original_err}");
    let err = mcb_domain::error::Error::vector_db(msg);
    let err_str = err.to_string();
    assert!(
        err_str.contains("Failed to query file paths"),
        "Error should mention query file paths: {err_str}"
    );
    assert!(
        err_str.contains("connection refused"),
        "Error should preserve original cause: {err_str}"
    );
}

#[test]
fn test_error_query_chunks_by_file_propagates_cause() {
    let collection = CollectionId::from_name("chunks-col");
    let original_err = "timeout";
    let msg =
        format!("Failed to query chunks by file in collection '{collection}': {original_err}");
    let err = mcb_domain::error::Error::vector_db(msg);
    let err_str = err.to_string();
    assert!(
        err_str.contains("Failed to query chunks by file"),
        "Error should mention query chunks by file: {err_str}"
    );
    assert!(
        err_str.contains("timeout"),
        "Error should preserve original cause: {err_str}"
    );
}

#[test]
fn test_error_collection_not_found_chunks_by_file() {
    let collection = CollectionId::from_name("missing-col");
    let err = mcb_domain::error::Error::vector_db(format!(
        "Collection '{collection}' not found when querying chunks by file"
    ));
    let err_str = err.to_string();
    assert!(
        err_str.contains("not found") && err_str.contains("chunks by file"),
        "Error should mention 'not found' and 'chunks by file': {err_str}"
    );
}
