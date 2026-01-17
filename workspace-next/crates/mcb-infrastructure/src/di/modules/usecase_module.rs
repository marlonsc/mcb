//! Use Case Module - Provides application use cases
//!
//! This module provides application use cases implementations.
//! Depends on context modules for external services.

use shaku::module;

// Import traits
use crate::di::modules::traits::UseCaseModule;

module! {
    pub UseCaseModuleImpl: UseCaseModule {
        components = [
            // Use cases created at runtime with dependencies
        ],
        providers = []
    }
}