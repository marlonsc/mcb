//! Golden tests contract and index.
//!
//! **Implemented tests live in `crates/mcb-server`** (no `#[ignore]`):
//! - `golden_tools_e2e.rs` (from release/v0.1.5)
//! - `golden_e2e_complete.rs` (E2E + index + MCP schema + search validation)
//! - `golden_acceptance_integration.rs`
//!
//! **Real binary E2E tests** (verified against real fixtures):
//! - `test_end_to_end.rs` — Complete workflow: clear → status → index → search
//! - `test_search_validation.rs` — Search relevance, ranking, filters, limits
//! - `test_index_repository.rs` — Index start/clear/status operations
//! - `test_mcp_schemas.rs` — MCP protocol schema validation
//! - `test_git_awareness_e2e.rs` — Git-aware indexing with .mcp-context.toml (v0.2.0)
//! - `test_memory_operations_e2e.rs` — Session memory + hooks (v0.2.0)
//!
//! Run: `cargo test -p mcb-server golden` or `make test SCOPE=golden`.
//!
//! Coverage: E2E workflow, index operations, search with filters/limits,
//! MCP schema (all tools), collection isolation, reindexing, performance baseline,
//! git-aware indexing, session memory operations.

mod test_end_to_end;
mod test_git_awareness_e2e;
mod test_index_repository;
mod test_mcp_schemas;
mod test_memory_operations_e2e;
mod test_search_validation;
