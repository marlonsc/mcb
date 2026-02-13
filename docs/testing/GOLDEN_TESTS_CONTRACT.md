<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
# Golden Tests Contract

Golden tests validate**real** MCP tool behaviour: indexing, search, status, and
clear. They run with the real DI stack (NullEmbedding + InMemoryVectorStore) and
assert on handler responses and content.

**Locations:** `crates/mcb-server/tests/integration/`
(`golden_e2e_complete.rs`, `golden_tools_e2e.rs`,
`golden_acceptance_integration.rs`) and `tests/golden/` for fixture data only
(non-executable).

**Run:** `cargo test -p mcb-server golden` or `make test SCOPE=golden`

**Fixtures:** `sample_codebase/` (Rust files), `golden_queries.json`
(queries + expected_files for E2E via handlers)

---

## 1. E2E workflow

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
| Test | Contract (what must hold) |
| ------ | --------------------------- |
| `golden_e2e_complete_workflow` | (1) index (action=clear)(collection) succeeds and response contains "clear"/"Clear"/"cleared". (2) index (action=status)(collection) succeeds, not error, text contains "Indexing Status" or "Idle" or "indexing". (3) index (action=start)(path, collection) succeeds, not error, text contains "chunks"/"file"/"Index"/"Files processed"/"Indexing Started". (4) index (action=status) again succeeds. (5) search (resource=code)(collection, query) succeeds, not error, text contains "Search"/"Results"/"Result". (6) index (action=clear) again succeeds. (7) index (action=status) again succeeds. |
| `golden_e2e_handles_concurrent_operations` | Two concurrent index (action=status)(collection) calls both succeed. |
| `golden_e2e_respects_collection_isolation` | index (action=clear)(collection_a) and index (action=clear)(collection_b) both succeed; operations on one collection do not break the other. |
| `golden_e2e_handles_reindex_correctly` | index (action=start)(path, collection) twice (reindex) both succeed; no panic, response indicates indexing (sync or async). |
| Golden queries E2E | See section 5 (split into setup, one query, all handlers succeed). |

---

## 2. Index

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
| Test | Contract |
| ------ | ---------- |
| `golden_index_test_repository` | index (action=start)(sample_codebase_path, collection) succeeds, not error, response content non-empty and contains "chunk"/"file"/"Index"/"Files processed"/"Indexing Started"/"Source directory"/"Path:". |
| `golden_index_handles_multiple_languages` | index (action=start) with extensions=Some(["rs"]) succeeds. |
| `golden_index_respects_ignore_patterns` | index (action=start) with ignore_patterns=Some(["*.md"]) succeeds. |

---

## 3. MCP response schema (content shape)

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
| Test | Contract |
| ------ | ---------- |
| `golden_mcp_index (action=start)_schema` | index (action=start) response: Ok, content non-empty, not is_error. |
| `golden_mcp_search (resource=code)_schema` | search (resource=code) response: Ok, content non-empty. |
| `golden_mcp_index (action=status)_schema` | index (action=status) response: Ok, content non-empty, text contains "Status"/"indexing"/"Idle". |
| `golden_mcp_index (action=clear)_schema` | index (action=clear) response: Ok, not error, text contains "Clear"/"clear"/"Collection"/"cleared". |
| `golden_mcp_error_responses_consistent` | search (resource=code) with empty query yields Err (validation error). |

---

## 4. Search validation

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
| Test | Contract |
| ------ | ---------- |
| `golden_search_returns_relevant_results` | After indexing sample_codebase into collection, search (resource=code)(collection, "embedding vector") succeeds, not error. (With null embedding, results may be empty; at least the handler must succeed.) |
| `golden_search_ranking_is_correct` | search (resource=code)(collection, query) succeeds. |
| `golden_search_handles_empty_query` | search (resource=code) with query "" or whitespace-only yields Err. |
| `golden_search_respects_limit_parameter` | search (resource=code) with limit=2 succeeds; response should reflect limit (e.g. "Results found: N" with N ≤ 2, or "Showing top 2 results"). |
| `golden_search_filters_by_extension` | search (resource=code) with extensions=Some(["rs"]) succeeds. |

---

## 5. Golden queries E2E (split to avoid timeout)

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
| Test | Contract |
| ------ | ---------- |
| `golden_e2e_golden_queries_setup` | index (action=clear), index (action=start), then poll index (action=status) until Idle/processed (bounded wait: 20 × 50ms). |
| `golden_e2e_golden_queries_one_query` | After clear + index, one search (resource=code) call succeeds and response is not error. |
| `golden_e2e_golden_queries_all_handlers_succeed` | After clear + index, run all queries from golden_queries.JSON; every search (resource=code) must succeed (no error). With null embedding, Result counts may be 0. |

---

## Implementation notes

- All tests use `create_test_mcp_server()` (null embedding + in-memory vector store).
- Indexing may return "Indexing Started" (async) or "Indexing Completed"
  (sync); assertions accept both.
- Search Result format: "**Results found:** N", "**1.** 📁 `path` (line L)",
  "Relevance Score"; use these to assert counts and file paths when
  strengthening tests.
- Golden-queries E2E is split into three tests (setup, one query, all handlers
  succeed) to keep each test short and avoid timeouts.
