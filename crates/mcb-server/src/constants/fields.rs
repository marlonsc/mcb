//! JSON response field name constants.
//!
//! Shared field names used across handler response formatting.
//! Domain-level schema keys live in `mcb_domain::constants::keys`.

/// JSON field: observation ID.
pub const FIELD_OBSERVATION_ID: &str = "observation_id";

/// JSON field: observation type.
pub const FIELD_OBSERVATION_TYPE: &str = "observation_type";

/// JSON field: session ID (response context).
pub const FIELD_SESSION_ID: &str = "session_id";

/// JSON field: message text.
pub const FIELD_MESSAGE: &str = "message";

/// JSON field: error event name.
pub const FIELD_ERROR: &str = "error";

/// JSON field: success/failure tag.
pub const TAG_SUCCESS: &str = "success";

/// JSON field: failure tag.
pub const TAG_FAILURE: &str = "failure";

/// JSON field: tool activity tag.
pub const TAG_TOOL: &str = "tool";

/// JSON field: quality gate tag.
pub const TAG_QUALITY_GATE: &str = "quality_gate";

/// JSON field: execution tag.
pub const TAG_EXECUTION: &str = "execution";
