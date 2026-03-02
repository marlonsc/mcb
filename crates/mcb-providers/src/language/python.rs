//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Python language processor for AST-based code chunking.

use mcb_utils::constants::ast::TS_NODE_FUNCTION_DEFINITION;
use mcb_utils::constants::lang::CHUNK_SIZE_PYTHON;

crate::impl_simple_language_processor!(
    PythonProcessor,
    language = tree_sitter_python::LANGUAGE.into(),
    chunk_size = CHUNK_SIZE_PYTHON,
    max_depth = 2,
    nodes = [TS_NODE_FUNCTION_DEFINITION, "class_definition"]
);
