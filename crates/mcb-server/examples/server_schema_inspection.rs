//! Prints generated JSON schemas for selected MCP argument enums.

use mcb_server::args::{IndexAction, SearchResource, ValidateAction};
use schemars::schema_for;

fn main() {
    println!("=== IndexAction (with rename_all = snake_case) ===");
    let index_schema = schema_for!(IndexAction);
    match serde_json::to_string_pretty(&index_schema) {
        Ok(json) => println!("{json}"),
        Err(err) => eprintln!("failed to render IndexAction schema: {err}"),
    }

    println!("\n=== SearchResource (with rename_all = snake_case) ===");
    let search_schema = schema_for!(SearchResource);
    match serde_json::to_string_pretty(&search_schema) {
        Ok(json) => println!("{json}"),
        Err(err) => eprintln!("failed to render SearchResource schema: {err}"),
    }

    println!("\n=== ValidateAction (with rename_all = snake_case) ===");
    let validate_schema = schema_for!(ValidateAction);
    match serde_json::to_string_pretty(&validate_schema) {
        Ok(json) => println!("{json}"),
        Err(err) => eprintln!("failed to render ValidateAction schema: {err}"),
    }
}
