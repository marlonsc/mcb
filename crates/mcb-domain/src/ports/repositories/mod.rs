//! Repository ports for data persistence.

pub mod memory_repository;
pub mod project_repository;

pub use memory_repository::MemoryRepository;
pub use project_repository::ProjectRepository;
