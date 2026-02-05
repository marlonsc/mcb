# Test Repository Fixture

This is a minimal test repository used for golden acceptance tests in MCB.

## Purpose

This repository contains sample code in multiple languages to test:

-   Multi-language indexing (Rust, Python, JavaScript)
-   Semantic search across different file types
-   Code chunking strategies
-   MCP tool functionality

## Contents

-   `src/calculator.rs` - Rust arithmetic functions
-   `src/string_utils.py` - Python String utilities
-   `src/utils.js` - JavaScript helper functions

## Expected Search Queries

The following queries should return relevant results:

| Query | Expected File |
|-------|---------------|
| "function that adds numbers" | calculator.rs |
| "reverse a String" | String_utils.py |
| "format a date" | utils.js |
| "check if palindrome" | String_utils.py |
| "debounce function" | utils.js |

## Usage in Golden Tests

This fixture is used by tests in `tests/golden/`:

-   `test_index_repository.rs` - Indexes this repo
-   `test_search_validation.rs` - Searches with known queries
-   `test_end_to_end.rs` - Complete workflow test
