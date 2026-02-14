//! Unit tests for CodeChunk entity
//!
//! Tests the core domain entity for code chunks, ensuring proper
//! creation, validation, and business rule enforcement.

#[cfg(test)]
mod tests {
    use mcb_domain::CodeChunk;
    use rstest::*;
    use serde_json::json;

    #[rstest]
    #[case("test-chunk-001", "fn hello() {}", "src/main.rs", 1, 3, "rust", json!({"type": "function"}))]
    fn test_code_chunk_creation(
        #[case] id: &str,
        #[case] content: &str,
        #[case] file_path: &str,
        #[case] start_line: u32,
        #[case] end_line: u32,
        #[case] language: &str,
        #[case] metadata: serde_json::Value,
    ) {
        let chunk = CodeChunk {
            id: id.to_string(),
            content: content.to_string(),
            file_path: file_path.to_string(),
            start_line,
            end_line,
            language: language.to_string(),
            metadata: metadata.clone(),
        };

        assert_eq!(chunk.id, id);
        assert_eq!(chunk.content, content);
        assert_eq!(chunk.file_path, file_path);
        assert_eq!(chunk.start_line, start_line);
        assert_eq!(chunk.end_line, end_line);
        assert_eq!(chunk.language, language);
        assert_eq!(chunk.metadata, metadata);
    }

    #[rstest]
    fn test_code_chunk_metadata_scenarios() {
        // Empty metadata
        let empty_chunk = CodeChunk {
            id: "empty".to_string(),
            content: "".to_string(),
            file_path: "".to_string(),
            start_line: 0,
            end_line: 0,
            language: "".to_string(),
            metadata: json!({}),
        };
        assert!(empty_chunk.metadata.as_object().unwrap().is_empty());

        // Complex metadata
        let complex_chunk = CodeChunk {
            id: "complex".to_string(),
            content: "".to_string(),
            file_path: "".to_string(),
            start_line: 0,
            end_line: 0,
            language: "".to_string(),
            metadata: json!({
                "type": "class",
                "methods": ["a", "b"]
            }),
        };
        assert_eq!(complex_chunk.metadata["type"], "class");
        assert!(complex_chunk.metadata["methods"].is_array());
    }
}
