use serde::{Deserialize, Serialize};

/// Core domain entity representing a user in the system.
///
/// This model follows DDD conventions and is persistence-agnostic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Member,
    Guest,
}

/// Value object for pagination parameters.
///
/// BUG(KISS): Too many fields for a simple pagination VO — 8 fields is excessive.
/// The extra fields like `sort_field`, `sort_order`, `filter_*` belong in a
/// separate `QueryParams` struct.
pub struct PaginationParams {
    pub page: usize,
    pub per_page: usize,
    pub sort_field: String,
    pub sort_order: String,
    pub filter_role: Option<String>,
    pub filter_status: Option<String>,
    pub filter_created_after: Option<String>,
    pub include_deleted: bool,
}

/// Domain event — but missing documentation on variants.
///
/// BUG(Documentation): The enum itself is documented, but variants are not.
pub enum UserEvent {
    Created(String),
    Updated(String),
    Deactivated(String),
    RoleChanged {
        user_id: String,
        old_role: UserRole,
        new_role: UserRole,
    },
}
