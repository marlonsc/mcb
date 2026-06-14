//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Centralized test constants and fixture values for the entire workspace.
//!
//! All test crates MUST import shared test constants from here instead of
//! defining them locally. This avoids drift between "test-org" in one crate
//! and "org-test" in another.

// ============================================================================
// Identity Constants
// ============================================================================

/// Default test organization ID.
pub const TEST_ORG_ID: &str = "test-org";

/// Organization A identifier for multi-tenant tests.
pub const TEST_ORG_ID_A: &str = "test-org-a";

/// Organization B identifier for multi-tenant tests.
pub const TEST_ORG_ID_B: &str = "test-org-b";

/// Default test project ID.
pub const TEST_PROJECT_ID: &str = "test-project";

/// Default test session ID.
pub const TEST_SESSION_ID: &str = "test-session";

/// Default test user email.
pub const TEST_USER_EMAIL: &str = "test@example.com";

/// Default test user ID string.
pub const TEST_USER_ID: &str = "test-user";

/// Default test repository name.
pub const TEST_REPO_NAME: &str = "test-repo";

/// Default test display name.
pub const TEST_DISPLAY_NAME: &str = "Test User";

/// Default test admin display name.
pub const TEST_ADMIN_DISPLAY_NAME: &str = "Test Admin";

// ============================================================================
// Timestamp / Time Constants
// ============================================================================

/// Default test timestamp (`2023-11-14T22:13:20Z`).
pub const TEST_TIMESTAMP: i64 = 1_700_000_000;

/// Generic small timestamp for fixture `created_at` / `updated_at` fields.
pub const TEST_TIMESTAMP_SMALL: i64 = 1000;

// ============================================================================
// Embedding / Dimension Constants
// ============================================================================

/// Default embedding dimensions for tests (`FastEmbed` MiniLM-L6-v2).
pub const TEST_EMBEDDING_DIMENSIONS: usize = 384;

// ============================================================================
// Collection / Resource Names
// ============================================================================

/// Default golden-test collection name.
pub const GOLDEN_COLLECTION: &str = "mcb_golden_test";

/// Expected files in `sample_codebase` for search assertions.
pub const SAMPLE_CODEBASE_FILES: &[&str] = &[
    "embedding.rs",
    "vector_store.rs",
    "handlers.rs",
    "cache.rs",
    "di.rs",
    "error.rs",
    "chunking.rs",
];

// ============================================================================
// Timeout Constants
// ============================================================================

/// Default timeout for integration/e2e tests (seconds).
pub const TEST_TIMEOUT_SECS: u64 = 30;

/// Startup timeout for MCP server integration tests (seconds).
pub const TEST_STARTUP_TIMEOUT_SECS: u64 = 120;

/// Operation timeout for MCP command tests (seconds).
pub const TEST_OP_TIMEOUT_SECS: u64 = 10;

// ============================================================================
// Test Model / Provider Names
// ============================================================================

/// Default test model name for agent sessions.
pub const TEST_MODEL_NAME: &str = "claude-sonnet";

/// Default test tool name for tool call fixtures.
pub const TEST_TOOL_NAME: &str = "test_tool";

// ============================================================================
// Test Fixture Strings
// ============================================================================

/// Default test project path.
pub const TEST_PROJECT_PATH: &str = "/tmp/test-project";

/// Default test prompt summary.
pub const TEST_PROMPT_SUMMARY: &str = "Test Prompt";

/// Default test phase description.
pub const TEST_PHASE_DESCRIPTION: &str = "Test Phase Description";

/// Default test issue description.
pub const TEST_ISSUE_DESCRIPTION: &str = "Test Issue Description";

/// Default test checkpoint description.
pub const TEST_CHECKPOINT_DESCRIPTION: &str = "Test Checkpoint";

/// Default test org settings JSON.
pub const TEST_SETTINGS_JSON: &str = "{}";

/// Default API key scopes JSON.
pub const TEST_API_KEY_SCOPES: &str = r#"["read", "write"]"#;
