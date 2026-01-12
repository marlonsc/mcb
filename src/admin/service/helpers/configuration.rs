//! Configuration history management
//!
//! Provides persistence for configuration changes, enabling audit trails
//! and history viewing in the admin interface.

use crate::admin::service::helpers::admin_defaults;
use crate::admin::service::types::{AdminError, ConfigurationChange};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Get the path to the configuration history file
fn history_file_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".context").join("config_history.json")
}

/// Configuration history store
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigHistory {
    pub entries: Vec<ConfigurationChange>,
}

/// Thread-safe configuration history manager
pub struct ConfigHistoryManager {
    history: RwLock<ConfigHistory>,
}

impl ConfigHistoryManager {
    /// Create a new history manager, loading existing history from disk
    pub async fn new() -> Result<Self, AdminError> {
        let history = load_history().await.unwrap_or_default();
        Ok(Self {
            history: RwLock::new(history),
        })
    }

    /// Record a configuration change
    pub async fn record_change(
        &self,
        user: &str,
        path: &str,
        change_type: &str,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
    ) -> Result<ConfigurationChange, AdminError> {
        let change = ConfigurationChange {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            user: user.to_string(),
            path: path.to_string(),
            old_value,
            new_value,
            change_type: change_type.to_string(),
        };

        let mut history = self.history.write().await;
        history.entries.insert(0, change.clone());

        // Trim to max entries
        if history.entries.len() > admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES {
            history.entries.truncate(admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES);
        }

        // Persist to disk (fire and forget, don't block)
        let history_clone = history.clone();
        tokio::spawn(async move {
            if let Err(e) = save_history(&history_clone).await {
                tracing::warn!("Failed to persist config history: {}", e);
            }
        });

        Ok(change)
    }

    /// Record multiple changes from a batch update
    pub async fn record_batch(
        &self,
        user: &str,
        updates: &HashMap<String, serde_json::Value>,
        old_config: Option<&HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<ConfigurationChange>, AdminError> {
        let mut changes = Vec::new();

        for (path, new_value) in updates {
            let old_value = old_config.and_then(|c| c.get(path).cloned());
            let change_type = if old_value.is_some() {
                "updated"
            } else {
                "added"
            };

            let change = ConfigurationChange {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                user: user.to_string(),
                path: path.clone(),
                old_value,
                new_value: new_value.clone(),
                change_type: change_type.to_string(),
            };
            changes.push(change);
        }

        // Add all changes to history
        {
            let mut history = self.history.write().await;
            for change in &changes {
                history.entries.insert(0, change.clone());
            }

            // Trim to max entries
            if history.entries.len() > admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES {
                history.entries.truncate(admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES);
            }

            // Persist
            let history_clone = history.clone();
            tokio::spawn(async move {
                if let Err(e) = save_history(&history_clone).await {
                    tracing::warn!("Failed to persist config history: {}", e);
                }
            });
        }

        Ok(changes)
    }

    /// Get configuration history with optional limit
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<ConfigurationChange> {
        let history = self.history.read().await;
        let limit = limit.unwrap_or(100);
        history.entries.iter().take(limit).cloned().collect()
    }

    /// Get total number of history entries
    pub async fn count(&self) -> usize {
        self.history.read().await.entries.len()
    }

    /// Clear all history (for testing or admin reset)
    pub async fn clear(&self) -> Result<(), AdminError> {
        let mut history = self.history.write().await;
        history.entries.clear();

        save_history(&history).await?;
        Ok(())
    }
}

/// Load history from disk
async fn load_history() -> Result<ConfigHistory, AdminError> {
    let path = history_file_path();

    if !path.exists() {
        return Ok(ConfigHistory::default());
    }

    let content = fs::read_to_string(&path)
        .await
        .map_err(|e| AdminError::InternalError(format!("Failed to read history file: {}", e)))?;

    serde_json::from_str(&content)
        .map_err(|e| AdminError::InternalError(format!("Failed to parse history file: {}", e)))
}

/// Save history to disk
async fn save_history(history: &ConfigHistory) -> Result<(), AdminError> {
    let path = history_file_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await.map_err(|e| {
            AdminError::InternalError(format!("Failed to create history directory: {}", e))
        })?;
    }

    let content = serde_json::to_string_pretty(history)
        .map_err(|e| AdminError::InternalError(format!("Failed to serialize history: {}", e)))?;

    fs::write(&path, content)
        .await
        .map_err(|e| AdminError::InternalError(format!("Failed to write history file: {}", e)))?;

    Ok(())
}

/// Standalone function to get configuration history (for use without manager instance)
pub async fn get_configuration_history(
    limit: Option<usize>,
) -> Result<Vec<ConfigurationChange>, AdminError> {
    let history = load_history().await?;
    let limit = limit.unwrap_or(100);
    Ok(history.entries.into_iter().take(limit).collect())
}

/// Standalone function to record a configuration change
pub async fn record_configuration_change(
    user: &str,
    path: &str,
    change_type: &str,
    old_value: Option<serde_json::Value>,
    new_value: serde_json::Value,
) -> Result<ConfigurationChange, AdminError> {
    let mut history = load_history().await?;

    let change = ConfigurationChange {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        user: user.to_string(),
        path: path.to_string(),
        old_value,
        new_value,
        change_type: change_type.to_string(),
    };

    history.entries.insert(0, change.clone());

    // Trim to max entries
    if history.entries.len() > admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES {
        history.entries.truncate(admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES);
    }

    save_history(&history).await?;

    Ok(change)
}

/// Standalone function to record batch configuration changes
pub async fn record_batch_changes(
    user: &str,
    updates: &HashMap<String, serde_json::Value>,
    old_config: Option<&HashMap<String, serde_json::Value>>,
) -> Result<Vec<ConfigurationChange>, AdminError> {
    let mut history = load_history().await?;
    let mut changes = Vec::new();

    for (path, new_value) in updates {
        let old_value = old_config.and_then(|c| c.get(path).cloned());
        let change_type = if old_value.is_some() {
            "updated"
        } else {
            "added"
        };

        let change = ConfigurationChange {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            user: user.to_string(),
            path: path.clone(),
            old_value,
            new_value: new_value.clone(),
            change_type: change_type.to_string(),
        };

        history.entries.insert(0, change.clone());
        changes.push(change);
    }

    // Trim to max entries
    if history.entries.len() > admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES {
        history.entries.truncate(admin_defaults::DEFAULT_MAX_HISTORY_ENTRIES);
    }

    save_history(&history).await?;

    Ok(changes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_and_get_history() {
        let manager = ConfigHistoryManager::new().await.unwrap();

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
    }

    #[tokio::test]
    async fn test_history_limit() {
        let manager = ConfigHistoryManager::new().await.unwrap();

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
    }
}
