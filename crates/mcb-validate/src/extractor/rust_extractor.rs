use super::fact::{Fact, FactType, Location};
use anyhow::Result;
use rust_code_analysis::{ParserTrait, RustParser};
use std::fs;
use std::path::Path;

pub struct RustExtractor;

impl RustExtractor {
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
            .to_string();

        let module_id = format!("module::{}", module_name);

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

        // Find "use" declarations (Imports)
        let imports = root.find_all(|n| n.kind() == "use_declaration");
        for import in imports {
            if let Some(text) = import.utf8_text(code_ref) {
                // Remove "use " and ";"
                let import_path = text.trim_start_matches("use ").trim_end_matches(';').trim();

                facts.push(Fact::new(
                    import_path.to_string(),
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
        let structs = root.find_all(|n| n.kind() == "struct_item");
        for st in structs {
            if let Some(name_node) = st.child_by_field_name("name") {
                if let Some(name) = name_node.utf8_text(code_ref) {
                    facts.push(Fact::new(
                        name.to_string(),
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
        }

        // Find Functions
        let functions = root.find_all(|n| n.kind() == "function_item");
        for func in functions {
            if let Some(name_node) = func.child_by_field_name("name") {
                if let Some(name) = name_node.utf8_text(code_ref) {
                    facts.push(Fact::new(
                        name.to_string(),
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
        }

        Ok(facts)
    }
}
