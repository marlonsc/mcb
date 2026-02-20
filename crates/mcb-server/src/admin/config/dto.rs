//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Configuration DTOs

use serde::{Deserialize, Serialize};

use super::view::SanitizedConfig;
use crate::constants::VALID_SECTIONS;

/// Configuration response (sanitized for API output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Configuration data (sanitized)
    pub config: SanitizedConfig,
    /// Configuration file path
    pub config_path: Option<String>,
    /// Last reload timestamp (RFC 3339)
    pub last_reload: Option<String>,
}

/// Configuration reload response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigReloadResponse {
    /// Whether the reload was successful
    pub success: bool,
    /// Reload result message
    pub message: String,
    /// New configuration (sanitized, if reload succeeded)
    pub config: Option<SanitizedConfig>,
    /// Reload timestamp (RFC 3339)
    pub reloaded_at: Option<String>,
}

impl ConfigReloadResponse {
    /// Create a success response
    #[must_use]
    pub fn success(config: SanitizedConfig) -> Self {
        Self {
            success: true,
            message: "Configuration reloaded successfully".to_owned(),
            config: Some(config),
            reloaded_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// Create a failure response
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            config: None,
            reloaded_at: None,
        }
    }

    /// Create a response indicating the watcher is not available
    #[must_use]
    pub fn watcher_unavailable() -> Self {
        Self {
            success: false,
            message: "Configuration watcher is not enabled".to_owned(),
            config: None,
            reloaded_at: None,
        }
    }
}

/// Configuration section update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSectionUpdateRequest {
    /// Section-specific configuration values to update
    pub values: serde_json::Value,
}

/// Configuration section update response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSectionUpdateResponse {
    /// Whether the update was successful
    pub success: bool,
    /// Update result message
    pub message: String,
    /// Section name that was updated
    pub section: String,
    /// Updated configuration (sanitized)
    pub config: Option<SanitizedConfig>,
    /// Update timestamp (RFC 3339)
    pub updated_at: Option<String>,
}

impl ConfigSectionUpdateResponse {
    /// Create a success response
    pub fn success(section: impl Into<String>, config: SanitizedConfig) -> Self {
        Self {
            success: true,
            message: "Configuration section updated successfully".to_owned(),
            section: section.into(),
            config: Some(config),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// Create a failure response
    pub fn failure(section: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            section: section.into(),
            config: None,
            updated_at: None,
        }
    }

    /// Create a response for invalid section
    pub fn invalid_section(section: impl Into<String>) -> Self {
        let section_name = section.into();
        Self {
            success: false,
            message: format!("Unknown configuration section: {section_name}"),
            section: section_name,
            config: None,
            updated_at: None,
        }
    }

    /// Create a response indicating the watcher is not available
    pub fn watcher_unavailable(section: impl Into<String>) -> Self {
        Self {
            success: false,
            message: "Configuration watcher is not enabled".to_owned(),
            section: section.into(),
            config: None,
            updated_at: None,
        }
    }
}

/// Check if a section name is valid for updates
#[must_use]
pub fn is_valid_section(section: &str) -> bool {
    VALID_SECTIONS.contains(&section)
}
