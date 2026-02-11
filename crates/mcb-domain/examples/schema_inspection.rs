use mcb_domain::entities::OrgStatus;
use schemars::schema_for;

fn main() {
    println!("=== OrgStatus (with JsonSchema derive) ===");
    let org_schema = schema_for!(OrgStatus);
    println!("{}", serde_json::to_string_pretty(&org_schema).unwrap());
}
