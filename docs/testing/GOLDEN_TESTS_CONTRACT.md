<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Golden Tests Contract

**Total test count: 118 tests across 14 sections**

Golden tests validate**real** MCP tool behaviour: indexing, search, status, and
clear. They run with the real DI stack (FastEmbedProvider + EdgeVecVectorStoreProvider) and
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
| `golden_search_returns_relevant_results` | After indexing sample_codebase into collection, search (resource=code)(collection, "embedding vector") succeeds, not error. (With FastEmbed, results may vary; at least the handler must succeed.) |
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
| `golden_e2e_golden_queries_all_handlers_succeed` | After clear + index, run all queries from golden_queries.JSON; every search (resource=code) must succeed (no error). Result counts may vary by embedding provider. |

---

## Implementation notes

- All tests use `create_test_mcp_server()` (FastEmbed + EdgeVec vector store).
- Indexing may return "Indexing Started" (async) or "Indexing Completed"
  (sync); assertions accept both.
- Search Result format: "**Results found:** N", "**1.** 📁 `path` (line L)",
  "Relevance Score"; use these to assert counts and file paths when
  strengthening tests.
- Golden-queries E2E is split into three tests (setup, one query, all handlers
  succeed) to keep each test short and avoid timeouts.

---

## 6. Org Entity CRUD (19 tests)

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
| Test | Contract |
| ------ | ---------- |
| `golden_org_create_and_get` | org_entity (action=create, resource=org) with data succeeds; org_entity (action=get, resource=org) with id succeeds; response contains id and name. |
| `golden_org_list` | org_entity (action=list, resource=org) succeeds; response is array with at least 2 orgs. |
| `golden_org_update` | org_entity (action=update, resource=org) with updated data succeeds; get after update reflects changes. |
| `golden_org_delete` | org_entity (action=delete, resource=org) succeeds; get after delete fails. |
| `golden_org_create_missing_data` | org_entity (action=create, resource=org) without data fails. |
| `golden_user_create_and_get` | org_entity (action=create, resource=user) with data succeeds; get returns user with id, email, org_id. |
| `golden_user_get_by_email` | org_entity (action=get, resource=user) with email succeeds; returns matching user. |
| `golden_user_list_by_org` | org_entity (action=list, resource=user) with org_id succeeds; returns array of users for that org. |
| `golden_user_update` | org_entity (action=update, resource=user) with updated data succeeds; get reflects changes. |
| `golden_user_delete` | org_entity (action=delete, resource=user) succeeds; get after delete fails. |
| `golden_user_create_missing_data` | org_entity (action=create, resource=user) without data fails. |
| `golden_user_get_missing_id_and_email` | org_entity (action=get, resource=user) without id/email fails with appropriate error. |
| `golden_team_create_and_get` | org_entity (action=create, resource=team) with data succeeds; get returns team with id. |
| `golden_team_list` | org_entity (action=list, resource=team) with org_id succeeds; returns array with at least 2 teams. |
| `golden_team_delete` | org_entity (action=delete, resource=team) succeeds; get after delete fails. |
| `golden_team_update_unsupported` | org_entity (action=update, resource=team) returns unsupported error. |
| `golden_team_member_add_and_list` | org_entity (action=create, resource=team_member) succeeds; list returns members including newly added. |
| `golden_team_member_remove` | org_entity (action=delete, resource=team_member) succeeds; list after delete no longer includes member. |
| `golden_team_member_get_unsupported` | org_entity (action=get, resource=team_member) returns unsupported error. |

---

## 7. Data Isolation (6 tests)

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
| Test | Contract |
| ------ | ---------- |
| `golden_isolation_users_scoped_to_org` | Users created in org-A do not appear in org-B user list. |
| `golden_isolation_teams_scoped_to_org` | Teams created in org-A do not appear in org-B team list. |
| `golden_isolation_api_keys_scoped_to_org` | API keys created in org-A do not appear in org-B api key list. |
| `golden_isolation_org_a_invisible_to_org_b` | Full scenario: users, teams, and keys in org-A are invisible to org-B lists. |
| `golden_isolation_cross_org_get_fails` | Attempting to get a user from org-A using org-B context returns empty or error. |
| `golden_isolation_both_orgs_coexist` | Both orgs can coexist; each lists only its own users. |

---

## 8. API Key Lifecycle (9 tests)

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
| Test | Contract |
| ------ | ---------- |
| `golden_api_key_create_and_get` | org_entity (action=create, resource=api_key) with data succeeds; get returns key with id, user_id, org_id, name, key_hash. |
| `golden_api_key_list_by_org` | org_entity (action=list, resource=api_key) with org_id succeeds; returns array of keys for that org. |
| `golden_api_key_revoke` | org_entity (action=update, resource=api_key) with revoked_at timestamp succeeds; get reflects revoked_at. |
| `golden_api_key_delete` | org_entity (action=delete, resource=api_key) succeeds; get after delete fails. |
| `golden_api_key_create_with_scopes` | org_entity (action=create, resource=api_key) with scopes_json succeeds; get returns scopes. |
| `golden_api_key_create_with_expiration` | org_entity (action=create, resource=api_key) with expires_at succeeds; get returns expires_at. |
| `golden_api_key_revoke_sets_timestamp` | org_entity (action=update, resource=api_key) with revoked_at sets timestamp > 0. |
| `golden_api_key_create_missing_data` | org_entity (action=create, resource=api_key) without data fails. |
| `golden_api_key_full_lifecycle` | Create → list (1 key) → revoke → list (1 revoked) → delete → list (0 keys). |

---

## 9. Session Lifecycle (6 tests)

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
| Test | Contract |
| ------ | ---------- |
| `golden_session_create_and_get` | session (action=create) with data succeeds; get returns session with id, status=active, agent_type, model, started_at. |
| `golden_session_list` | session (action=list) succeeds; returns array with at least 2 sessions and count field. |
| `golden_session_end` | session (action=update) with status=completed succeeds; get reflects completed status. |
| `golden_session_create_missing_data` | session (action=create) without data returns error or is_error=true. |
| `golden_session_get_nonexistent` | session (action=get) with fake id returns error or 'not found' text. |
| `golden_session_summary` | session (action=summarize) with data succeeds; response contains summary_id or session_id. |

---

## 10. VCS Entity CRUD (10 tests)

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
| Test | Contract |
| ------ | ---------- |
| `golden_vcs_repo_create_and_get` | vcs_entity (action=create, resource=repository) with data succeeds; get returns repo with id, name, url. |
| `golden_vcs_repo_list` | vcs_entity (action=list, resource=repository) with org_id and project_id succeeds; returns array with at least 2 repos. |
| `golden_vcs_repo_update` | vcs_entity (action=update, resource=repository) with updated data succeeds; get reflects changes. |
| `golden_vcs_repo_delete` | vcs_entity (action=delete, resource=repository) succeeds; get after delete fails. |
| `golden_vcs_branch_create_and_get` | vcs_entity (action=create, resource=branch) with data succeeds; get returns branch with id, name, repository_id. |
| `golden_vcs_branch_list` | vcs_entity (action=list, resource=branch) with repository_id succeeds; returns array with at least 2 branches. |
| `golden_vcs_branch_delete` | vcs_entity (action=delete, resource=branch) succeeds; get after delete fails. |
| `golden_vcs_worktree_create_and_get` | vcs_entity (action=create, resource=worktree) with data succeeds; get returns worktree with id, repository_id, branch_id. |
| `golden_vcs_worktree_list` | vcs_entity (action=list, resource=worktree) with repository_id succeeds; returns array with at least 2 worktrees. |
| `golden_vcs_worktree_delete` | vcs_entity (action=delete, resource=worktree) succeeds; get after delete fails. |

---

## 11. Plan Entity CRUD (12 tests)

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
| Test | Contract |
| ------ | ---------- |
| `golden_plan_create_and_get` | plan_entity (action=create, resource=plan) with data succeeds; get returns plan with id, title, project_id. |
| `golden_plan_list` | plan_entity (action=list, resource=plan) with project_id succeeds; returns array with at least 2 plans. |
| `golden_plan_update` | plan_entity (action=update, resource=plan) with updated data succeeds; get reflects changes. |
| `golden_plan_delete` | plan_entity (action=delete, resource=plan) succeeds; get after delete fails. |
| `golden_plan_version_create_and_get` | plan_entity (action=create, resource=version) with data succeeds; get returns version with id, plan_id, version_number. |
| `golden_plan_version_list` | plan_entity (action=list, resource=version) with plan_id succeeds; returns array with at least 2 versions. |
| `golden_plan_version_delete` | plan_entity (action=delete, resource=version) either unsupported or succeeds; get after delete fails if supported. |
| `golden_plan_review_create_and_get` | plan_entity (action=create, resource=review) with data succeeds; get returns review with id, plan_version_id, verdict. |
| `golden_plan_review_list` | plan_entity (action=list, resource=review) with plan_version_id succeeds; returns array with at least 2 reviews. |
| `golden_plan_review_delete` | plan_entity (action=delete, resource=review) either unsupported or succeeds; get after delete fails if supported. |
| `golden_plan_create_missing_data` | plan_entity (action=create, resource=plan) without data fails. |
| `golden_plan_get_nonexistent` | plan_entity (action=get, resource=plan) with fake id fails. |

---

## 12. Issue Entity CRUD (13 tests)

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
| Test | Contract |
| ------ | ---------- |
| `golden_issue_create_and_get` | issue_entity (action=create, resource=issue) with data succeeds; get returns issue with id, title, project_id. |
| `golden_issue_list` | issue_entity (action=list, resource=issue) with project_id succeeds; returns array with at least 2 issues. |
| `golden_issue_update_status` | issue_entity (action=update, resource=issue) with status change succeeds; get reflects new status. |
| `golden_issue_update_assignee` | issue_entity (action=update, resource=issue) with assignee change succeeds; get reflects new assignee. |
| `golden_issue_delete` | issue_entity (action=delete, resource=issue) succeeds; get after delete fails. |
| `golden_issue_comment_create_and_get` | issue_entity (action=create, resource=comment) with data succeeds; get returns comment with id, content, issue_id. |
| `golden_issue_comment_list` | issue_entity (action=list, resource=comment) with issue_id succeeds; returns array with at least 2 comments. |
| `golden_issue_comment_delete` | issue_entity (action=delete, resource=comment) succeeds; get after delete fails. |
| `golden_issue_label_create_and_get` | issue_entity (action=create, resource=label) with data succeeds; get returns label with id, name, project_id. |
| `golden_issue_label_list` | issue_entity (action=list, resource=label) with project_id succeeds; returns array with at least 2 labels. |
| `golden_issue_label_assign_to_issue` | issue_entity (action=create, resource=label_assignment) succeeds; list returns assignment with label_id. |
| `golden_issue_create_missing_fields` | issue_entity (action=create, resource=issue) without data fails. |
| `golden_issue_get_nonexistent` | issue_entity (action=get, resource=issue) with fake id fails. |

---

## 13. Validate Operations (4 tests)

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
| Test | Contract |
| ------ | ---------- |
| `golden_validate_analyze` | validate (action=analyze, scope=file) with valid file path succeeds; response is not error. |
| `golden_validate_status` | validate (action=list_rules) succeeds; response is not error and contains validators/count. |
| `golden_validate_missing_path` | validate (action=run, scope=file) with nonexistent path returns Ok with is_error=true. |
| `golden_validate_empty_args` | validate (action=analyze) without path returns MCP error. |

---

## 14. Project Operations (4 tests)

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
| Test | Contract |
| ------ | ---------- |
| `golden_project_create_phase` | project (action=create, resource=phase) returns unsupported error (not yet implemented). |
| `golden_project_list_phases` | project (action=list, resource=phase) returns unsupported error (not yet implemented). |
| `golden_project_create_decision` | project (action=create, resource=decision) returns unsupported error (not yet implemented). |
| `golden_project_missing_project_id` | project (action=get, resource=project) without project_id returns error mentioning project_id is required. |
