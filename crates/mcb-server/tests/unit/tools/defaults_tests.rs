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

#[rstest]
#[case("git@github.com:owner/repo.git", Some(("owner", "repo")))]
#[case("github-neptor:owner/repo.git", Some(("owner", "repo")))]
#[case("https://github.com/owner/repo.git", Some(("owner", "repo")))]
#[case("https://github.com/owner/repo", Some(("owner", "repo")))]
#[case("git@gitlab.com:group/sub/repo.git", Some(("group", "sub/repo")))]
#[case("file:///tmp/local.git", None)]
#[case("/absolute/path", None)]
#[case("https://github.com/onlyorg", None)]
#[case("not-a-remote", None)]
fn remote_url_parsing_accepts_canonical_and_alias_forms(
    #[case] url: &str,
    #[case] expected: Option<(&str, &str)>,
) {
    use mcb_server::tools::defaults::parse_org_and_project_from_remote_url;
    let parsed = parse_org_and_project_from_remote_url(url);
    assert_eq!(
        parsed,
        expected.map(|(o, p)| (o.to_owned(), p.to_owned()))
    );
}
