use mcb_server::args::{IndexAction, SearchResource, ValidateAction};
use schemars::schema_for;

fn main() {
    println!("=== IndexAction (with rename_all = snake_case) ===");
    let index_schema = schema_for!(IndexAction);
    println!("{}", serde_json::to_string_pretty(&index_schema).unwrap());

    println!("\n=== SearchResource (with rename_all = snake_case) ===");
    let search_schema = schema_for!(SearchResource);
    println!("{}", serde_json::to_string_pretty(&search_schema).unwrap());

    println!("\n=== ValidateAction (with rename_all = snake_case) ===");
    let validate_schema = schema_for!(ValidateAction);
    println!(
        "{}",
        serde_json::to_string_pretty(&validate_schema).unwrap()
    );
}
