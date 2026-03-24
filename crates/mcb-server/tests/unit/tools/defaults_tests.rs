//! Tests for `mcb_server::tools::defaults`.

use mcb_server::tools::defaults::ExecutionFlow;
use std::str::FromStr;

#[test]
fn execution_flow_round_trip_stdio() {
    let flow = ExecutionFlow::StdioOnly;
    let parsed = ExecutionFlow::from_str(flow.as_str()).expect("valid flow");
    assert_eq!(parsed, flow);
}

#[test]
fn execution_flow_round_trip_hybrid() {
    let flow = ExecutionFlow::ClientHybrid;
    let parsed = ExecutionFlow::from_str(flow.as_str()).expect("valid flow");
    assert_eq!(parsed, flow);
}

#[test]
fn execution_flow_round_trip_server_hybrid() {
    let flow = ExecutionFlow::ServerHybrid;
    let parsed = ExecutionFlow::from_str(flow.as_str()).expect("valid flow");
    assert_eq!(parsed, flow);
}

#[test]
fn execution_flow_invalid_returns_error() {
    let result = ExecutionFlow::from_str("invalid_mode");
    assert!(result.is_err());
}

#[test]
fn execution_flow_display() {
    let flow = ExecutionFlow::StdioOnly;
    assert_eq!(flow.to_string(), flow.as_str());
}
