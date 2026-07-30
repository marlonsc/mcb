//! Prints the generated JSON schema for `OrgStatus`.

use std::io::Write;

use mcb_domain::entities::OrgStatus;
use schemars::schema_for;

fn main() {
    let _ = writeln!(
        std::io::stdout(),
        "=== OrgStatus (with JsonSchema derive) ==="
    );
    let org_schema = schema_for!(OrgStatus);
    match serde_json::to_string_pretty(&org_schema) {
        Ok(json) => {
            let _ = writeln!(std::io::stdout(), "{json}");
        }
        Err(err) => {
            let _ = writeln!(std::io::stderr(), "failed to render schema: {err}");
        }
    }
}
