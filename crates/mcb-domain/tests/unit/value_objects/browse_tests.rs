//! Unit tests for browse value objects.

use mcb_domain::value_objects::{CollectionId, CollectionInfo, FileInfo};
use rstest::rstest;

#[rstest]
fn test_collection_info_new() {
    let info = CollectionInfo::new(
        "test-collection",
        100,
        10,
        Some(1705680000),
        mcb_utils::constants::PROVIDER_SLUG_MILVUS,
    );

    assert_eq!(info.name, "test-collection");
    assert_eq!(info.id, CollectionId::from_name("test-collection"));
    assert_eq!(info.vector_count, 100);
    assert_eq!(info.file_count, 10);
    assert_eq!(info.last_indexed, Some(1705680000));
    assert_eq!(info.provider, mcb_utils::constants::PROVIDER_SLUG_MILVUS);
}

#[rstest]
fn test_collection_info_serialization() {
    let info = CollectionInfo::new("test", 50, 5, None, "in_memory");
    let json = serde_json::to_string(&info).expect("serialization should succeed");
    let deserialized: CollectionInfo =
        serde_json::from_str(&json).expect("deserialization should succeed");

    assert_eq!(info, deserialized);
}

#[rstest]
fn test_file_info_new() {
    let info = FileInfo::new("src/main.rs", 5, "rust", Some(1024));

    assert_eq!(info.path, "src/main.rs");
    assert_eq!(info.chunk_count, 5);
    assert_eq!(info.language, "rust");
    assert_eq!(info.size_bytes, Some(1024));
}

#[rstest]
fn test_file_info_serialization() {
    let info = FileInfo::new("lib.rs", 3, "rust", None);
    let json = serde_json::to_string(&info).expect("serialization should succeed");
    let deserialized: FileInfo =
        serde_json::from_str(&json).expect("deserialization should succeed");

    assert_eq!(info, deserialized);
}

// =============================================================================
// map_highlight_to_category tests
// =============================================================================
use mcb_domain::value_objects::browse::{
    HIGHLIGHT_NAMES, HighlightCategory, map_highlight_to_category,
};

#[rstest]
#[case("keyword", HighlightCategory::Keyword)]
#[case("string", HighlightCategory::String)]
#[case("comment", HighlightCategory::Comment)]
#[case("function", HighlightCategory::Function)]
#[case("variable", HighlightCategory::Variable)]
#[case("constant", HighlightCategory::Variable)]
#[case("attribute", HighlightCategory::Variable)]
#[case("property", HighlightCategory::Variable)]
#[case("tag", HighlightCategory::Variable)]
#[case("type", HighlightCategory::Type)]
#[case("number", HighlightCategory::Number)]
#[case("operator", HighlightCategory::Operator)]
#[case("punctuation", HighlightCategory::Punctuation)]
fn test_highlight_names_map_correctly(#[case] name: &str, #[case] expected: HighlightCategory) {
    assert_eq!(map_highlight_to_category(name), expected);
}

#[rstest]
#[case("punctuation.bracket")]
#[case("punctuation.delimiter")]
#[case("punctuation.special")]
fn test_punctuation_variants_map_to_punctuation(#[case] name: &str) {
    assert_eq!(
        map_highlight_to_category(name),
        HighlightCategory::Punctuation
    );
}

#[rstest]
#[case("unknown_token")]
#[case("")]
#[case("foobar")]
fn test_unknown_names_map_to_other(#[case] name: &str) {
    assert_eq!(map_highlight_to_category(name), HighlightCategory::Other);
}

#[rstest]
fn test_all_highlight_names_have_non_other_category() {
    for name in &HIGHLIGHT_NAMES {
        let category = map_highlight_to_category(name);
        assert_ne!(
            category,
            HighlightCategory::Other,
            "HIGHLIGHT_NAMES entry '{name}' should map to a specific category, not Other"
        );
    }
}
