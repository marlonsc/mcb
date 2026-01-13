//! Infrastructure utility modules - Modularized from the original 1143-line utils.rs
//!
//! Organized by responsibility:
//! - validation: Input validation and validation configuration
//! - formatting: Data formatting (numbers, bytes, duration, age, percentages)
//! - collections: Collection utilities (safe access, defaults)
//! - file_io: Async file I/O with error handling
//! - http: HTTP response checking and JSON parsing
//! - retry: Async retry with exponential backoff
//! - json: JSON value extension trait with convenient accessors
//! - css: CSS class constants for UI components
//! - status: Status utilities and constants
//! - strings: String manipulation (title case, capitalization, relative time)
//! - time: Time utilities (Unix timestamps, expiration checks)
//! - provider_utils: Provider type inference and parsing

pub mod collections;
pub mod css;
pub mod file_io;
pub mod formatting;
pub mod http;
pub mod json;
pub mod locks;
pub mod provider_utils;
pub mod retry;
pub mod status;
pub mod strings;
pub mod time;
pub mod validation;

// Re-export public API for backward compatibility
pub use collections::CollectionUtils;
pub use file_io::FileUtils;
pub use formatting::FormattingUtils;
pub use http::{HttpResponseUtils, IntoStatusCode};
pub use json::JsonExt;
pub use locks::{AsyncRwLockExt, RwLockExt};
pub use provider_utils::ProviderUtils;
pub use retry::{RetryConfig, RetryUtils};
pub use status::{HealthUtils, StatusUtils};
pub use strings::StringUtils;
pub use time::TimeUtils;
pub use validation::{
    ConfigurableValidator, StringValidator, Validatable, ValidationConfig, ValidationResult,
};

// Re-export nested modules for direct access
pub use css::{badge_for_level, badge_for_status, indicator_for_level, indicator_for_status};
pub use status::activity_level;
