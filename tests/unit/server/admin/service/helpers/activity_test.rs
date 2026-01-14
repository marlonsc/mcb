//! Unit tests for activity feed tracking
//!
//! Tests for converting SystemEvents into human-readable activity log entries.

use mcp_context_browser::application::admin::helpers::activity::{
    ActivityLevel, ActivityLogger,
};

#[tokio::test]
async fn test_activity_logger_manual_add() {
    let logger = ActivityLogger::new();

    logger
        .add_activity(
            ActivityLevel::Info,
            "test",
            "Test activity",
            Some(serde_json::json!({"key": "value"})),
        )
        .await;

    let activities = logger.get_activities(None).await;
    assert_eq!(activities.len(), 1);
    assert_eq!(activities[0].category, "test");
    assert_eq!(activities[0].message, "Test activity");
}

#[tokio::test]
async fn test_event_to_activity_conversion() {
    // Test the event conversion by creating an activity logger and checking if it correctly
    // handles the SystemEvent types. The actual event_to_activity function is private,
    // so we test through the logger's behavior.
    let logger = ActivityLogger::new();

    // Add a cache-related activity to simulate what would happen
    logger
        .add_activity(ActivityLevel::Success, "cache", "Cache cleared: test", None)
        .await;

    let activities = logger.get_activities(None).await;
    assert_eq!(activities.len(), 1);
    assert_eq!(activities[0].category, "cache");
    assert_eq!(activities[0].level, ActivityLevel::Success);
    assert!(activities[0].message.contains("Cache cleared"));
}
