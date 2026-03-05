use mcb_domain::CodeChunk;
use rstest::{fixture, rstest};
use serde_json::json;

#[fixture]
fn default_chunk() -> CodeChunk {
    CodeChunk {
        id: "test-chunk-001".to_owned(),
        content: "fn hello() {}".to_owned(),
        file_path: "src/main.rs".to_owned(),
        start_line: 1,
        end_line: 3,
        language: "rust".to_owned(),
        metadata: json!({"type": "function"}),
    }
}

#[rstest]
fn test_code_chunk_construction(default_chunk: CodeChunk) {
    assert_eq!(default_chunk.id, "test-chunk-001");
    assert_eq!(default_chunk.language, "rust");
}

#[rstest]
fn test_code_chunk_metadata_scenarios(mut default_chunk: CodeChunk) {
    // Empty metadata
    default_chunk.metadata = json!({});
    assert!(default_chunk.metadata.as_object().unwrap().is_empty());

    // Complex metadata
    default_chunk.metadata = json!({
        "type": "class",
        "methods": ["a", "b"]
    });
    assert_eq!(default_chunk.metadata["type"], "class");
    assert!(default_chunk.metadata["methods"].is_array());
}

#[rstest]
#[case("c1", "content1")]
#[case("c2", "content2")]
fn test_code_chunk_variants(#[case] id: &str, #[case] content: &str, mut default_chunk: CodeChunk) {
    default_chunk.id = id.to_owned();
    default_chunk.content = content.to_owned();
    assert_eq!(default_chunk.id, id);
    assert_eq!(default_chunk.content, content);
}
