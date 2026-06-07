//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! C# language processor for AST-based code chunking.

use crate::language::common::CHUNK_SIZE_CSHARP;
use mcb_domain::constants::ast::{
    AST_NODE_INTERFACE_DECLARATION, TS_NODE_CLASS_DECLARATION, TS_NODE_METHOD_DECLARATION,
};

crate::impl_simple_language_processor!(
    CSharpProcessor,
    language = tree_sitter_c_sharp::LANGUAGE.into(),
    chunk_size = CHUNK_SIZE_CSHARP,
    max_depth = 3,
    nodes = [
        TS_NODE_METHOD_DECLARATION,
        TS_NODE_CLASS_DECLARATION,
        AST_NODE_INTERFACE_DECLARATION
    ]
);
