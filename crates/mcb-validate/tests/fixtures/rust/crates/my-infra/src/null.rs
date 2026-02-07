//! Null object implementations — stubs for testing/development.
//!
//! Files named `null.rs` are EXEMPT from empty-method checks because
//! they intentionally provide no-op implementations.

/// Null implementation of the user repository.
///
/// Returns empty results for all operations — safe for use in tests
/// and development where a real database is not available.
pub struct NullUserRepository;

impl NullUserRepository {
    /// Returns None — this is intentionally empty (null object pattern).
    pub fn find_by_id(&self, _id: &str) -> Option<String> {
        None
    }

    /// Does nothing — this is intentionally empty (null object pattern).
    pub fn save(&self, _data: &str) -> Result<(), String> {
        Ok(())
    }

    /// Does nothing — null delete is a no-op.
    pub fn delete(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }
}
