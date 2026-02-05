//! Golden tests contract and index.
//!
//! **Implemented tests live in `crates/mcb-server`** (no `#[ignore]`):
//! - `golden_tools_e2e.rs` (from release/v0.1.5)
//! - `golden_e2e_complete.rs` (E2E + index + MCP schema + search validation)
//! - `golden_acceptance_integration.rs`
//!
//! **Real binary E2E tests in `tests/golden/`**:
//! - `test_real_binary_e2e.rs` (Multi-language, complex searches, performance)
//! - `test_end_to_end.rs` (Complete workflow)
//! - `test_search_validation.rs` (Search relevance, filters)
//! - `test_index_repository.rs` (Index operations)
//! - `test_mcp_schemas.rs` (MCP protocol)
//!
//! Run: `cargo test -p mcb-server golden` or `make test SCOPE=golden`.
//!
//! Coverage: E2E workflow, index (repo/multi-lang/ignore), search (relevance/limit/ext),
//! MCP schema (all 4 tools + error format), collection isolation, reindex, performance.

mod test_real_binary_e2e;
