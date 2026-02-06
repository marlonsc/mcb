//! HTTP Client Abstractions re-exported from domain
//!
//! This module provides backward compatibility for providers using HTTP clients.
//! Canonical traits are now defined in mcb-domain.

pub use crate::utils::HttpResponseUtils;
pub use mcb_domain::ports::providers::http::{HttpClientConfig, HttpClientProvider};
