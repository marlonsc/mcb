//! Golden acceptance tests for MCB
//!
//! **Implemented tests live in `crates/mcb-server`** (no `#[ignore]`).
//!
//! Run them with:
//! - `cargo test -p mcb-server golden` (acceptance + tools e2e)
//! - `make test SCOPE=golden`
//!
//! Coverage:
//! - E2E: complete workflow (clear → index → status → search → clear)
//! - Index: test repository, multiple languages, ignore patterns
//! - Search: relevance, ranking, empty query, limit, extension filter
//! - MCP schema: index_codebase, search_code, get_indexing_status, clear_index, error format
//! - Collection isolation and reindex
