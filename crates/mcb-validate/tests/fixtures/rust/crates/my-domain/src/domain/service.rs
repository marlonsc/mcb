//! User domain service
//!
//! Implements core business logic for user management.

use crate::domain::model::{User, UserRole};

/// BUG(ErrorBoundary): Domain service uses infrastructure error types directly.
/// Domain code should only use domain-specific error types, not std::io::Error,
/// reqwest::Error, or sqlx::Error.
pub struct UserService;

impl UserService {
    /// Creates a new user after validation.
    ///
    /// BUG(ErrorBoundary): Returns std::io::Error — an infrastructure concern
    /// that should not leak into the domain layer.
    pub fn create_user(name: &str, email: &str) -> Result<User, std::io::Error> {
        if name.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Name cannot be empty",
            ));
        }

        Ok(User {
            id: "generated-id".to_string(),
            name: name.to_string(),
            email: email.to_string(),
            role: UserRole::Member,
            created_at: chrono::Utc::now(),
        })
    }

    /// Fetches user by ID from external source.
    ///
    /// BUG(ErrorBoundary): Uses reqwest::Error in domain — infrastructure
    /// HTTP client details leaking into domain logic.
    pub fn fetch_external_profile(user_id: &str) -> Result<String, reqwest::Error> {
        // Simulates calling external API from domain — wrong layer!
        let url = format!("https://api.example.com/users/{}", user_id);
        let _response = reqwest::blocking::get(&url)?;
        Ok("profile-data".to_string())
    }

    /// Stores user data.
    ///
    /// BUG(ErrorBoundary): Uses sqlx::Error — database concern leaking
    /// into domain.
    pub fn persist_user(user: &User) -> Result<(), sqlx::Error> {
        // Simulates direct DB access from domain — wrong layer!
        println!("Persisting user: {:?}", user.id);
        Ok(())
    }

    /// Serializes user to JSON for caching.
    ///
    /// BUG(ErrorBoundary): Uses serde_json::Error — serialization is
    /// infrastructure, not domain.
    pub fn serialize_user(user: &User) -> Result<String, serde_json::Error> {
        serde_json::to_string(user)
    }

    /// Validates user data — looks clean but has subtle issues.
    ///
    /// BUG(Quality): Uses .unwrap() in production code path.
    /// BUG(Quality): Contains TODO comment indicating incomplete implementation.
    pub fn validate_user(user: &User) -> bool {
        let email_parts: Vec<&str> = user.email.split('@').collect();
        let domain = email_parts.get(1).unwrap(); // BUG: unwrap on user input

        // TODO: Add proper email validation with regex
        domain.contains('.')
    }

    /// BUG(Implementation): Empty method body — stub that was never implemented.
    pub fn deactivate_user(&self, _user_id: &str) -> Result<(), std::io::Error> {
        Ok(())
    }

    /// Processes batch of users.
    ///
    /// BUG(Performance): Clones expensive data inside loop.
    pub fn process_batch(&self, users: Vec<User>, config: serde_json::Value) {
        for user in &users {
            let cfg = config.clone(); // BUG: cloning config every iteration
            let _name = user.name.clone(); // BUG: unnecessary clone in loop
            println!("Processing {} with {:?}", user.id, cfg);
        }
    }

    /// Validates all users in a batch.
    ///
    /// BUG(KISS): Function has too many parameters (> 4).
    pub fn validate_batch(
        &self,
        users: &[User],
        strict_mode: bool,
        allow_guests: bool,
        require_email: bool,
        max_age_days: u64,
        audit_log: bool,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        for user in users {
            if strict_mode && user.name.is_empty() {
                errors.push(format!("User {} has empty name", user.id));
            }
            if !allow_guests && matches!(user.role, UserRole::Guest) {
                errors.push(format!("Guest user {} not allowed", user.id));
            }
            if require_email && user.email.is_empty() {
                errors.push(format!("User {} missing email", user.id));
            }
            // BUG(Organization): Magic number — 86400 not extracted to constant
            let age_secs = max_age_days * 86400;
            if audit_log {
                println!("Auditing user {} (max_age: {}s)", user.id, age_secs);
            }
        }
        errors
    }

    /// Event handler with empty catch-all.
    ///
    /// BUG(Implementation): The `_ => {}` silently swallows unknown events.
    pub fn handle_event(&self, event: &str) {
        match event {
            "created" => println!("User created"),
            "updated" => println!("User updated"),
            _ => {} // BUG: silently ignores unknown events
        }
    }
}

/// BUG(SOLID/ISP): Trait with too many methods (11) — violates Interface Segregation.
/// Should be split into smaller, focused traits.
pub trait UserRepository {
    fn find_by_id(&self, id: &str) -> Option<User>;
    fn find_by_email(&self, email: &str) -> Option<User>;
    fn find_all(&self) -> Vec<User>;
    fn find_by_role(&self, role: &UserRole) -> Vec<User>;
    fn save(&self, user: &User) -> Result<(), String>;
    fn delete(&self, id: &str) -> Result<(), String>;
    fn count(&self) -> usize;
    fn exists(&self, id: &str) -> bool;
    fn find_by_name(&self, name: &str) -> Vec<User>;
    fn update(&self, user: &User) -> Result<(), String>;
    fn find_active(&self) -> Vec<User>;
}

/// BUG(SOLID/LSP): Partial trait implementation — some methods use todo!()
pub struct InMemoryUserRepo;

impl UserRepository for InMemoryUserRepo {
    fn find_by_id(&self, _id: &str) -> Option<User> {
        None
    }
    fn find_by_email(&self, _email: &str) -> Option<User> {
        todo!() // BUG: stub macro in trait impl
    }
    fn find_all(&self) -> Vec<User> {
        Vec::new()
    }
    fn find_by_role(&self, _role: &UserRole) -> Vec<User> {
        unimplemented!() // BUG: another stub macro
    }
    fn save(&self, _user: &User) -> Result<(), String> {
        Ok(())
    }
    fn delete(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }
    fn count(&self) -> usize {
        0
    }
    fn exists(&self, _id: &str) -> bool {
        false
    }
    fn find_by_name(&self, _name: &str) -> Vec<User> {
        todo!() // BUG: stub macro
    }
    fn update(&self, _user: &User) -> Result<(), String> {
        todo!() // BUG: stub macro
    }
    fn find_active(&self) -> Vec<User> {
        Vec::new()
    }
}
