//! Unit test suite for mcb-providers
//!
//! Run with: `cargo test -p mcb-providers --test unit --features hybrid-search`

mod hybrid_search_tests;

mod submodule_tests;

mod git2_provider_tests;

mod cargo_tests;

mod go_tests;

mod maven_tests;

mod npm_tests;

mod python_tests;

mod ddl_tests;

mod project_repository_tests;

mod agent_repository_tests;

mod transitions_tests;

mod org_entity_repository_tests;

mod vcs_entity_repository_tests;

mod plan_entity_repository_tests;

mod issue_entity_repository_tests;

mod native_analysis_tests;

mod schema_upgrade_rejection_tests;

mod entity_test_utils;
