//! Indexing Service Use Case
//!
//! # Overview
//! The `IndexingService` manages the ingestion and processing of code assets into the semantic
//! context system. It handles the full lifecycle from file discovery to vector storage, ensuring
//! that the system's understanding of the codebase remains up-to-date.
//!
//! # Responsibilities
//! - **File Discovery**: Recursively scanning workspace directories while respecting ignore patterns.
//! - **Language-Aware Chunking**: Splitting code files into semantic chunks using AST-based strategies.
//! - **Incremental Indexing**: Optimizing ingestion by only processing changed files (via hash tracking).
//! - **Async Processing**: Executing long-running indexing tasks in the background to maintain responsiveness.
//! - **Event Publishing**: Notifying the system of indexing progress and completion.
//!
//! # Architecture
//! Implements `IndexingServiceInterface` and acts as a coordinator between:
//! - `LanguageChunkingProvider`: For parsing and splitting code.
//! - `ContextService`: For embedding and storing chunks.
//! - `FileHashRepository`: For change detection.
//! - `EventBusProvider`: For system-wide notifications.

mod discovery;
mod interface;
mod processing;
mod progress;
mod registry;
mod service;

pub use processing::*;
pub use progress::IndexingProgress;
pub use service::{
    IndexingServiceDeps, IndexingServiceImpl, IndexingServiceWithHashDeps, ProcessResult,
};
