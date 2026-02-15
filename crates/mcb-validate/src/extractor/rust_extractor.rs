//! Rust-specific AST fact extractor.
//!
//! Uses `rust-code-analysis` (which wraps `tree-sitter`) to parse Rust source
//! files and extract [`Fact`]s for modules, imports, structs, and functions.

use super::fact::{Fact, FactType, Location};
use crate::Result;
use rust_code_analysis::{Node, ParserTrait, RustParser};
use std::fs;
use std::path::Path;

/// Extracts code facts from Rust source files via AST analysis.
pub struct RustExtractor;

impl RustExtractor {
    /// Parse a Rust file at `path` and return all extracted [`Fact`]s.
    ///
    /// Currently extracts:
    /// - One `Module` fact per file (derived from the file stem).
    /// - `Import` facts for every `use` declaration.
    /// - `Struct` facts for every `struct` item.
    /// - `Function` facts for every `fn` item.
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn extract_facts(&self, path: &Path) -> Result<Vec<Fact>> {
        let code = fs::read(path)?;
        // Use RustParser which is a public type alias for Parser<RustCode>
        let parser = RustParser::new(code, path, None);
        let root = parser.get_root();
        let code_ref = parser.get_code();

        let mut facts = Vec::new();

        // Detect module name from path
        let module_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_owned();

        let module_id = format!("module::{module_name}");

        // Add Module Fact
        facts.push(Fact::new(
            module_name.clone(),
            FactType::Module,
            Location {
                file_path: path.to_path_buf(),
                start_line: root.start_row() + 1,
                end_line: root.end_row() + 1,
                start_column: root.start_position().1 + 1,
                end_column: root.end_position().1 + 1,
            },
            None,
        ));

        // Helper for finding nodes by kind
        fn find_by_kind<'a>(root: Node<'a>, kind: &str) -> Vec<Node<'a>> {
            let mut results = Vec::new();
            let mut stack = vec![root.0]; // Access inner tree-sitter node

            while let Some(n) = stack.pop() {
                if n.kind() == kind {
                    results.push(Node(n));
                }

                let mut cursor = n.walk();
                for child in n.children(&mut cursor) {
                    stack.push(child);
                }
            }
            results
        }

        // Find "use" declarations (Imports)
        let imports = find_by_kind(root, "use_declaration");
        for import in imports {
            if let Some(text) = import.utf8_text(code_ref) {
                // Remove "use " and ";"
                let import_path = text.trim_start_matches("use ").trim_end_matches(';').trim();

                facts.push(Fact::new(
                    import_path.to_owned(),
                    FactType::Import,
                    Location {
                        file_path: path.to_path_buf(),
                        start_line: import.start_row() + 1,
                        end_line: import.end_row() + 1,
                        start_column: import.start_position().1 + 1,
                        end_column: import.end_position().1 + 1,
                    },
                    Some(module_id.clone()),
                ));
            }
        }

        // Find Structs
        let structs = find_by_kind(root, "struct_item");
        for st in structs {
            if let Some(name_node) = st.0.child_by_field_name("name")
                && let Ok(name) = name_node.utf8_text(code_ref)
            {
                facts.push(Fact::new(
                    name.to_owned(),
                    FactType::Struct,
                    Location {
                        file_path: path.to_path_buf(),
                        start_line: st.start_row() + 1,
                        end_line: st.end_row() + 1,
                        start_column: st.start_position().1 + 1,
                        end_column: st.end_position().1 + 1,
                    },
                    Some(module_id.clone()),
                ));
            }
        }

        // Find Functions
        let functions = find_by_kind(root, "function_item");
        for func in functions {
            if let Some(name_node) = func.0.child_by_field_name("name")
                && let Ok(name) = name_node.utf8_text(code_ref)
            {
                facts.push(Fact::new(
                    name.to_owned(),
                    FactType::Function,
                    Location {
                        file_path: path.to_path_buf(),
                        start_line: func.start_row() + 1,
                        end_line: func.end_row() + 1,
                        start_column: func.start_position().1 + 1,
                        end_column: func.end_position().1 + 1,
                    },
                    Some(module_id.clone()),
                ));
            }
        }

        Ok(facts)
    }
}
