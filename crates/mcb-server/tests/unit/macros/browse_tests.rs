use mcb_server::admin::browse::{list_browse_issues, list_browse_plans, list_browse_repositories};

#[test]
fn macro_generated_handlers_are_callable() {
    let _repos: fn(_, _, _) -> _ = list_browse_repositories;
    let _plans: fn(_, _, _) -> _ = list_browse_plans;
    let _issues: fn(_, _, _) -> _ = list_browse_issues;
}
