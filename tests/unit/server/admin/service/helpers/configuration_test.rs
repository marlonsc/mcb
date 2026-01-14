//! Unit tests for configuration history management
//!
//! Tests for persistence of configuration changes and audit trails.

use mcp_context_browser::application::admin::helpers::configuration::ConfigHistoryManager;

#[tokio::test]
async fn test_record_and_get_history() {
    let temp_dir = std::env::temp_dir();
    let history_path = temp_dir.join("test_config_history.json");
    let manager = ConfigHistoryManager::new(history_path.clone())
        .await
        .unwrap();

    // Record a change
    let change = manager
        .record_change(
            "test_user",
            "metrics.enabled",
            "updated",
            Some(serde_json::json!(false)),
            serde_json::json!(true),
        )
        .await
        .unwrap();

    assert_eq!(change.user, "test_user");
    assert_eq!(change.path, "metrics.enabled");
    assert_eq!(change.change_type, "updated");

    // Get history
    let history = manager.get_history(Some(10)).await;
    assert!(!history.is_empty());
    assert_eq!(history[0].id, change.id);

    // Clean up
    let _ = tokio::fs::remove_file(&history_path).await;
}

#[tokio::test]
async fn test_history_limit() {
    let temp_dir = std::env::temp_dir();
    let history_path = temp_dir.join("test_config_history_limit.json");
    let manager = ConfigHistoryManager::new(history_path.clone())
        .await
        .unwrap();

    // Record multiple changes
    for i in 0..5 {
        manager
            .record_change(
                "test_user",
                &format!("path.{}", i),
                "added",
                None,
                serde_json::json!(i),
            )
            .await
            .unwrap();
    }

    // Get limited history
    let history = manager.get_history(Some(3)).await;
    assert!(history.len() <= 3);

    // Clean up
    let _ = tokio::fs::remove_file(&history_path).await;
}
