//! Application DI Module Implementation
//!
//! Contains business logic services (ContextService, SearchService, IndexingService).

//!
//! ## Service Hierarchy
//!
//! Services depend on repositories from AdaptersModule:
//! - ContextService injects ChunkRepository, SearchRepository, EmbeddingProvider
//! - SearchService injects ContextServiceInterface
//! - IndexingService injects ContextServiceInterface, ChunkingOrchestratorInterface
//!
//! ## Module Dependencies
//!
//! ApplicationModule uses AdaptersModule as a submodule to provide:
//! - ChunkRepository (for ContextService)
//! - SearchRepository (for ContextService)
//! - EmbeddingProvider (for ContextService)

#![allow(missing_docs)]

use shaku::module;

use super::traits::{AdaptersModule, ApplicationModule};
use crate::application::context::ContextService;
use crate::application::indexing::{ChunkingOrchestrator, IndexingService};
use crate::application::search::SearchService;
use crate::domain::chunking::IntelligentChunker;
use crate::domain::ports::{ChunkRepository, EmbeddingProvider, SearchRepository};

module! {
    pub ApplicationModuleImpl: ApplicationModule {
        components = [
            ContextService,
            SearchService,
            IndexingService,
            ChunkingOrchestrator,
            IntelligentChunker
        ],
        providers = [],

        use dyn AdaptersModule {
            components = [dyn ChunkRepository, dyn SearchRepository, dyn EmbeddingProvider],
            providers = []
        }
    }
}
