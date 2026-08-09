//! Execution flow modes: stdio, client-hybrid, server-hybrid.
//!
//! Tests verify that flow mode strings survive a round-trip parse and that
//! invalid modes are rejected with a helpful error.

use mcb_server::tools::defaults::ExecutionFlow;
use rstest::rstest;
use std::str::FromStr;

#[rstest]
#[case(ExecutionFlow::StdioOnly)]
#[case(ExecutionFlow::ClientHybrid)]
#[case(ExecutionFlow::ServerHybrid)]
fn every_flow_mode_survives_serialization_round_trip(#[case] flow: ExecutionFlow) {
    let parsed = ExecutionFlow::from_str(flow.as_str()).expect("valid flow");
    assert_eq!(parsed, flow);
    assert_eq!(flow.to_string(), flow.as_str());
}

#[rstest]
#[case("invalid_mode")]
#[case("")]
#[case("STDIO")]
fn unrecognized_flow_mode_rejected_with_error(#[case] input: &str) {
    assert!(ExecutionFlow::from_str(input).is_err());
}
