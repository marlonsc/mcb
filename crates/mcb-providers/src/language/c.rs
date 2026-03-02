//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! C language processor for AST-based code chunking.

use crate::language::common::CHUNK_SIZE_C;
use mcb_domain::constants::ast::{AST_NODE_STRUCT_SPECIFIER, TS_NODE_FUNCTION_DEFINITION};

crate::impl_simple_language_processor!(
    CProcessor,
    language = tree_sitter_c::LANGUAGE.into(),
    chunk_size = CHUNK_SIZE_C,
    max_depth = 2,
    nodes = [TS_NODE_FUNCTION_DEFINITION, AST_NODE_STRUCT_SPECIFIER]
);
