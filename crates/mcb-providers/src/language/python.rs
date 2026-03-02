//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Python language processor for AST-based code chunking.

use crate::language::common::CHUNK_SIZE_PYTHON;
use mcb_domain::constants::ast::TS_NODE_FUNCTION_DEFINITION;

crate::impl_simple_language_processor!(
    PythonProcessor,
    language = tree_sitter_python::LANGUAGE.into(),
    chunk_size = CHUNK_SIZE_PYTHON,
    max_depth = 2,
    nodes = [TS_NODE_FUNCTION_DEFINITION, "class_definition"]
);
