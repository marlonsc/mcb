//! Prints generated JSON schemas for selected MCP argument enums.

use mcb_server::args::{IndexAction, SearchResource, ValidateAction};
use schemars::schema_for;
use std::io::{self, Write};

fn write_schema<W: Write>(
    out: &mut W,
    name: &str,
    value: &impl serde::Serialize,
) -> io::Result<()> {
    writeln!(out, "=== {name} ===")?;
    match serde_json::to_string_pretty(value) {
        Ok(json) => writeln!(out, "{json}"),
        Err(err) => writeln!(out, "failed to render {name}: {err}"),
    }
}

fn main() -> io::Result<()> {
    let mut out = io::stdout();
    let index_schema = schema_for!(IndexAction);
    write_schema(
        &mut out,
        "IndexAction (with rename_all = snake_case)",
        &index_schema,
    )?;
    writeln!(out)?;

    let search_schema = schema_for!(SearchResource);
    write_schema(
        &mut out,
        "SearchResource (with rename_all = snake_case)",
        &search_schema,
    )?;
    writeln!(out)?;

    let validate_schema = schema_for!(ValidateAction);
    write_schema(
        &mut out,
        "ValidateAction (with rename_all = snake_case)",
        &validate_schema,
    )
}
