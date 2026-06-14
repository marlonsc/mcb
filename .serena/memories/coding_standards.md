# MCB Coding Standards (Rust)

## Error Handling
- **NEVER** use `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()` in production paths
- Use `thiserror` constructors: `Error::vcs("msg")`
- Use `?` for propagation
- Prefer `Result` over panics

## Imports Order
1. `std`
2. External crates
3. `mcb_*` crates
4. Local modules

## Type Safety
- Most restrictive type that compiles
- No `Any`, bare `object`, or unchecked casts
- No suppression directives (`# type: ignore`, blanket `# noqa`)

## Code Style
- Source files should stay under ~200 lines; split modules before growing
- Use existing macros: `tool_action!`, `tool_schema!`, `tool_enum!`, `register_tool!`
- Generated docs and reports fixed at generator/template

## Lint Policy (from Cargo.toml)
**Denied**: `unsafe_code`, `dead_code`, `unused_imports`, `dbg_macro`, `todo`, `unimplemented`, `exit`, `rc_mutex`, `try_err`
**Warned**: `unwrap_used`, `expect_used`, `panic`, `print_stdout`, `print_stderr`

## Unsafe Code
- `unsafe_code = "deny"` at workspace level
- No exceptions without explicit ADR

## Testing
- Unit tests: `make test SCOPE=unit`
- Integration tests: `make test SCOPE=integration`
- Doctests: `make test SCOPE=doc`
- Use `mockall` for mocking, `rstest` for parameterized tests

## Documentation
- `missing_docs = "warn"` at workspace level
- Document `Result` error conditions (`missing_errors_doc`)
- Document panic conditions (`missing_panics_doc`)
- Use backticks for code in docs (`doc_markdown`)
