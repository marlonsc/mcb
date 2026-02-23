//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Ruby language processor for AST-based code chunking.

use crate::language::common::CHUNK_SIZE_RUBY;

crate::impl_simple_language_processor!(
    RubyProcessor,
    language = tree_sitter_ruby::LANGUAGE.into(),
    chunk_size = CHUNK_SIZE_RUBY,
    max_depth = 2,
    nodes = ["method", "class", "module"]
);
