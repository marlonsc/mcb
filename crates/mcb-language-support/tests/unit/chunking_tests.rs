//! Unit tests for code chunking
//!
//! Tests for `SemanticChunking` and `LineBasedChunking` functionality.

use std::path::Path;

use mcb_language_support::chunking::{
    ChunkType, ChunkingConfig, ChunkingStrategy, LineBasedChunking, SemanticChunking,
};
use mcb_language_support::language::LanguageId;
use rstest::*;

#[tokio::test]
async fn semantic_chunking_rust() {
    let chunker = SemanticChunking::default();
    let code = r#"fn foo() {
    println!("foo");
}

fn bar(x: i32) -> i32 {
    if x > 0 {
        x * 2
    } else {
        -x
    }
}
"#;

    let chunks = chunker
        .chunk(code, LanguageId::Rust, Path::new("test.rs"))
        .await
        .expect("Should chunk");

    // Should find the functions
    assert!(!chunks.is_empty());

    // All chunks should have valid line numbers
    for chunk in &chunks {
        assert!(chunk.start_line >= 1);
        assert!(chunk.end_line >= chunk.start_line);
    }
}

#[tokio::test]
async fn semantic_chunking_small_file() {
    let chunker = SemanticChunking::default();
    let code = "x = 1\ny = 2\n";

    let chunks = chunker
        .chunk(code, LanguageId::Python, Path::new("test.py"))
        .await
        .expect("Should chunk");

    // Small file should be treated as single module chunk
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].chunk_type, ChunkType::Module);
}

#[tokio::test]
async fn line_based_chunking() {
    let config = ChunkingConfig {
        target_lines: 3,
        ..Default::default()
    };
    let chunker = LineBasedChunking::new(config);

    let code = "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\n";

    let chunks = chunker
        .chunk(code, LanguageId::Python, Path::new("test.py"))
        .await
        .expect("Should chunk");

    // Should create multiple chunks of ~3 lines each
    assert!(chunks.len() >= 2);

    for chunk in &chunks {
        assert_eq!(chunk.chunk_type, ChunkType::Block);
    }
}

#[tokio::test]
async fn empty_content() {
    let chunker = SemanticChunking::default();
    let chunks = chunker
        .chunk("", LanguageId::Rust, Path::new("empty.rs"))
        .await
        .expect("Should handle empty");

    assert!(chunks.is_empty());
}

#[rstest]
#[case(ChunkType::Function, "function")]
#[case(ChunkType::Class, "class")]
#[case(ChunkType::Module, "module")]
#[case(ChunkType::Block, "block")]
fn chunk_type_display(#[case] chunk_type: ChunkType, #[case] expected: &str) {
    assert_eq!(chunk_type.to_string(), expected);
}
