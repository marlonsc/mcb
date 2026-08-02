//! Integration tests for external service detection helpers.
//!
//! Copyright (c) 2025 MCB Contributors. All rights reserved.
//! SPDX-License-Identifier: MIT

use mcb_domain::utils::tests::service_detection::{
    EXTERNAL_SERVICES, all_required_services_available, format_service_summary,
    service_availability_summary,
};
use mcb_domain::utils::tests::services_config::test_service_url;

#[test]
fn summary_lists_all_external_services() {
    let summary = service_availability_summary();
    let keys: Vec<_> = summary.iter().map(|s| s.key).collect();
    for (key, _name) in EXTERNAL_SERVICES {
        assert!(keys.contains(key), "missing service {key} in summary");
    }
}

#[test]
fn format_summary_contains_every_service_name() {
    let summary = service_availability_summary();
    let output = format_service_summary(&summary);
    assert!(output.starts_with("External service availability:"));
    for (_, name) in EXTERNAL_SERVICES {
        assert!(output.contains(name), "summary missing {name}");
    }
}

#[test]
fn all_required_services_available_with_empty_list() {
    assert!(all_required_services_available(&[]));
}

#[test]
fn all_required_services_available_reflects_config() {
    // The result depends on the environment; we only assert consistency:
    // if milvus is not configured, requiring it must return false.
    if test_service_url("milvus").is_none() {
        assert!(!all_required_services_available(&["milvus"]));
    }
}
