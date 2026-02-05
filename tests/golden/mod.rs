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
//!
//! Run: `cargo test -p mcb-server golden` or `make test SCOPE=golden`.
//!
//! Coverage: E2E workflow, index operations, search with filters/limits,
//! MCP schema (all tools), collection isolation, reindexing, performance baseline.
