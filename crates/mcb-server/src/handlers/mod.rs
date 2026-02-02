//! MCP Tool Handlers
//!
//! Implementations of MCP tool calls using domain services.
//! Each handler translates MCP protocol requests into domain service calls.

pub mod analyze_complexity;
pub mod clear_index;
pub mod get_indexing_status;
pub mod get_validation_rules;
pub mod index_codebase;
pub mod index_git_repository;
pub mod list_repositories;
pub mod list_validators;
pub mod search_branch;
pub mod search_code;
pub mod validate_architecture;
pub mod validate_file;

// Re-export handlers for convenience
pub use analyze_complexity::AnalyzeComplexityHandler;
pub use clear_index::ClearIndexHandler;
pub use get_indexing_status::GetIndexingStatusHandler;
pub use get_validation_rules::GetValidationRulesHandler;
pub use index_codebase::IndexCodebaseHandler;
pub use index_git_repository::IndexGitRepositoryHandler;
pub use list_repositories::ListRepositoriesHandler;
pub use list_validators::ListValidatorsHandler;
pub use search_branch::SearchBranchHandler;
pub use search_code::SearchCodeHandler;
pub use validate_architecture::ValidateArchitectureHandler;
pub use validate_file::ValidateFileHandler;
