//! `CodeChunk` test fixture builders.
//!
//! Centralized in `mcb-domain` since `CodeChunk` lives in the domain layer.

use crate::entities::CodeChunk;

/// Create a test `CodeChunk` with the given content, file path, and start line.
#[must_use]
pub fn create_test_chunk(content: &str, file_path: &str, start_line: u32) -> CodeChunk {
    CodeChunk {
        id: format!("{file_path}:{start_line}"),
        content: content.to_owned(),
        file_path: file_path.to_owned(),
        start_line,
        end_line: start_line + content.lines().count() as u32,
        language: "Rust".to_owned(),
        metadata: serde_json::json!({}),
    }
}

/// Create multiple test `CodeChunk`s with numbered content.
#[must_use]
pub fn create_test_chunks(count: usize) -> Vec<CodeChunk> {
    (0..count)
        .map(|i| {
            create_test_chunk(
                &format!("fn test_function_{i}() {{\n    // test code\n}}"),
                &format!("src/file_{i}.rs"),
                (i as u32) * 10 + 1,
            )
        })
        .collect()
}
