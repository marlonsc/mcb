//! Shared test context — re-exports Loco-style shared state for unit tests.
//!
//! All handler wiring goes through Loco; tests use [`shared_mcb_state`] built from
//! [`create_real_domain_services`] (no manual bootstrap).

pub use crate::utils::test_fixtures::{
    shared_app_context, shared_mcb_state, try_shared_app_context, try_shared_mcb_state,
};
