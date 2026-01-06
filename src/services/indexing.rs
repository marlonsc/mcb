//! Indexing service for processing codebases

use crate::error::{Error, Result};
use crate::services::context::ContextService;
use crate::types::CodeChunk;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Simple indexing service for MVP
pub struct IndexingService {
    context_service: std::sync::Arc<ContextService>,
}

impl IndexingService {
    /// Create a new indexing service
    pub fn new(context_service: std::sync::Arc<ContextService>) -> Self {
        Self { context_service }
    }

    /// Index a directory (simplified for MVP)
    pub async fn index_directory(&self, path: &Path, collection: &str) -> Result<usize> {
        if !path.exists() || !path.is_dir() {
            return Err(Error::not_found("Directory not found"));
        }

        let mut chunks = Vec::new();

        // Simple file discovery (just Rust files for MVP)
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "rs" {
                        match self.process_file(entry.path()).await {
                            Ok(file_chunks) => chunks.extend(file_chunks),
                            Err(e) => eprintln!("Failed to process {}: {}", entry.path().display(), e),
                        }
                    }
                }
            }
        }

        // Store chunks in vector database
        self.context_service.store_chunks(collection, &chunks).await?;

        Ok(chunks.len())
    }

    /// Process a single file into chunks
    async fn process_file(&self, path: &Path) -> Result<Vec<CodeChunk>> {
        let content = fs::read_to_string(path)?;
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Very simple chunking for MVP - split by lines
        let chunks: Vec<CodeChunk> = content.lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty())
            .map(|(i, line)| CodeChunk {
                id: format!("{}_{}", file_name, i),
                content: line.to_string(),
                file_path: path.display().to_string(),
                start_line: i as u32,
                end_line: i as u32,
                language: crate::types::Language::Rust,
                embedding: None,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("file".to_string(), serde_json::json!(file_name));
                    meta.insert("line".to_string(), serde_json::json!(i));
                    meta
                },
            })
            .collect();

        Ok(chunks)
    }
}