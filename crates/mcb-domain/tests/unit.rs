//! Unit test suite for mcb-domain
//!
//! Run with: `cargo test -p mcb-domain --test unit`

#[path = "unit/chunk_repository_tests.rs"]
mod chunk_repository_tests;

#[path = "unit/code_chunk_tests.rs"]
mod code_chunk;

#[path = "unit/codebase_tests.rs"]
mod codebase;

#[path = "unit/config_tests.rs"]
mod config;

#[path = "unit/constants_tests.rs"]
mod constants;

#[path = "unit/domain_events_tests.rs"]
mod domain_events;

#[path = "unit/embedding_tests.rs"]
mod embedding;

#[path = "unit/error_tests.rs"]
mod error;

#[path = "unit/search_tests.rs"]
mod search;

#[path = "unit/types_tests.rs"]
mod types;

#[path = "unit/browse_tests.rs"]
mod browse;

#[path = "unit/ports/providers/metrics_tests.rs"]
mod metrics;

#[path = "unit/vcs_context_tests.rs"]
mod vcs_context_tests;

#[path = "unit/memory_tests.rs"]
mod memory_tests;

#[path = "unit/project_tests.rs"]
mod project_tests;

#[path = "unit/submodule_tests.rs"]
mod submodule_tests;

#[path = "unit/vcs_tests.rs"]
mod vcs_tests;

#[path = "unit/agent_tests.rs"]
mod agent_tests;
#[path = "unit/workflow_tests.rs"]
mod workflow_tests;

#[path = "unit/ids_tests.rs"]
mod ids_tests;

#[path = "unit/api_key_tests.rs"]
mod api_key_tests;
#[path = "unit/organization_tests.rs"]
mod organization_tests;
#[path = "unit/team_tests.rs"]
mod team_tests;
#[path = "unit/user_tests.rs"]
mod user_tests;

#[path = "unit/issue_tests.rs"]
mod issue_tests;
#[path = "unit/plan_tests.rs"]
mod plan_tests;
#[path = "unit/repository_tests.rs"]
mod repository_tests;
#[path = "unit/worktree_tests.rs"]
mod worktree_tests;
