//! Prints the generated JSON schema for `OrgStatus`.

use mcb_domain::entities::OrgStatus;
use schemars::schema_for;

fn main() {
    println!("=== OrgStatus (with JsonSchema derive) ===");
    let org_schema = schema_for!(OrgStatus);
    match serde_json::to_string_pretty(&org_schema) {
        Ok(json) => println!("{json}"),
        Err(err) => eprintln!("failed to render schema: {err}"),
    }
}
