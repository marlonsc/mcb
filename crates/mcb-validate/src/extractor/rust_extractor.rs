//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
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
    fn location_for(path: &Path, node: &Node<'_>) -> Location {
        Location {
            file_path: path.to_path_buf(),
            start_line: node.start_row() + 1,
            end_line: node.end_row() + 1,
            start_column: node.start_position().1 + 1,
            end_column: node.end_position().1 + 1,
        }
    }

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
            .unwrap_or(mcb_utils::constants::FALLBACK_UNKNOWN)
            .to_owned();

        let module_id = format!("module::{module_name}");

        // Add Module Fact
        facts.push(Fact::new(
            module_name.clone(),
            FactType::Module,
            Self::location_for(path, &root),
            None,
        ));

        let ctx = FactCtx {
            path,
            code_ref,
            module_id: &module_id,
        };
        Self::collect_imports(&ctx, &root, &mut facts);
        Self::collect_named(&ctx, &root, "struct_item", &FactType::Struct, &mut facts);
        Self::collect_named(
            &ctx,
            &root,
            "function_item",
            &FactType::Function,
            &mut facts,
        );

        Ok(facts)
    }

    /// Collect every AST node of `kind` reachable from `root`.
    fn find_by_kind<'a>(root: &Node<'a>, kind: &str) -> Vec<Node<'a>> {
        let mut results = Vec::new();
        let mut stack = vec![root.0];

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

    /// Append an [`FactType::Import`] fact for each `use` declaration.
    fn collect_imports(ctx: &FactCtx, root: &Node, facts: &mut Vec<Fact>) {
        for import in Self::find_by_kind(root, "use_declaration") {
            if let Some(text) = import.utf8_text(ctx.code_ref) {
                let import_path = text.trim_start_matches("use ").trim_end_matches(';').trim();
                facts.push(Fact::new(
                    import_path.to_owned(),
                    FactType::Import,
                    Self::location_for(ctx.path, &import),
                    Some(ctx.module_id.to_owned()),
                ));
            }
        }
    }

    /// Append a named fact of `fact_type` for each named node of `kind`.
    fn collect_named(
        ctx: &FactCtx,
        root: &Node,
        kind: &str,
        fact_type: &FactType,
        facts: &mut Vec<Fact>,
    ) {
        for node in Self::find_by_kind(root, kind) {
            if let Some(name_node) = node.0.child_by_field_name("name")
                && let Ok(name) = name_node.utf8_text(ctx.code_ref)
            {
                facts.push(Fact::new(
                    name.to_owned(),
                    fact_type.clone(),
                    Self::location_for(ctx.path, &node),
                    Some(ctx.module_id.to_owned()),
                ));
            }
        }
    }
}

/// Shared context for fact collection within a single source file.
struct FactCtx<'a> {
    path: &'a Path,
    code_ref: &'a [u8],
    module_id: &'a str,
}
