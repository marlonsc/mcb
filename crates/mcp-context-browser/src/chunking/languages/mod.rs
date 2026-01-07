//! Language-specific processors and configurations
//!
//! This module contains all language-specific chunking configurations
//! and processors for supported programming languages.

use crate::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::chunking::processor::{BaseProcessor, LanguageProcessor};
use crate::core::types::{CodeChunk, Language};

// Rust processor
pub struct RustProcessor {
    processor: BaseProcessor,
}

impl Default for RustProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl RustProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_rust::LANGUAGE.into())
            .with_rules(vec![
                // High priority: Main constructs
                NodeExtractionRule {
                    node_types: vec![
                        "function_item".to_string(),
                        "struct_item".to_string(),
                        "enum_item".to_string(),
                        "impl_item".to_string(),
                        "trait_item".to_string(),
                    ],
                    min_length: 40,
                    min_lines: 2,
                    max_depth: 4,
                    priority: 10,
                    include_context: true,
                },
                // Medium priority: Modules and macros
                NodeExtractionRule {
                    node_types: vec![
                        "mod_item".to_string(),
                        "macro_definition".to_string(),
                        "const_item".to_string(),
                        "static_item".to_string(),
                    ],
                    min_length: 25,
                    min_lines: 1,
                    max_depth: 3,
                    priority: 5,
                    include_context: false,
                },
                // Low priority: Type aliases and use statements
                NodeExtractionRule {
                    node_types: vec!["type_item".to_string(), "use_declaration".to_string()],
                    min_length: 15,
                    min_lines: 1,
                    max_depth: 2,
                    priority: 1,
                    include_context: false,
                },
            ])
            .with_fallback_patterns(vec![
                r"^fn ".to_string(),
                r"^struct ".to_string(),
                r"^impl ".to_string(),
                r"^pub fn ".to_string(),
                r"^pub struct ".to_string(),
                r"^enum ".to_string(),
                r"^trait ".to_string(),
                r"^mod ".to_string(),
                r"^const ".to_string(),
                r"^static ".to_string(),
            ])
            .with_chunk_size(20);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(RustProcessor);

// Python processor
pub struct PythonProcessor {
    processor: BaseProcessor,
}

impl Default for PythonProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_python::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "function_definition".to_string(),
                    "class_definition".to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 2,
                priority: 5,
                include_context: true,
            }])
            .with_fallback_patterns(vec![r"^def ".to_string(), r"^class ".to_string()])
            .with_chunk_size(15);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(PythonProcessor);

// JavaScript/TypeScript processor
pub struct JavaScriptProcessor {
    processor: BaseProcessor,
}

impl Default for JavaScriptProcessor {
    fn default() -> Self {
        Self::new(Language::JavaScript)
    }
}

impl JavaScriptProcessor {
    pub fn new(language: Language) -> Self {
        let language_instance = match language {
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TSX.into(),
            _ => tree_sitter_javascript::LANGUAGE.into(),
        };
        let config = LanguageConfig::new(language_instance)
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "function_declaration".to_string(),
                    "function".to_string(),
                    "class_declaration".to_string(),
                    "method_definition".to_string(),
                    "arrow_function".to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 2,
                priority: 9,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^function ".to_string(),
                r"^const .*=>\s*\{".to_string(),
                r"^class ".to_string(),
            ])
            .with_chunk_size(15);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(JavaScriptProcessor);

// Java processor
pub struct JavaProcessor {
    processor: BaseProcessor,
}

impl Default for JavaProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_java::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "method_declaration".to_string(),
                    "class_declaration".to_string(),
                    "interface_declaration".to_string(),
                    "constructor_declaration".to_string(),
                ],
                min_length: 40,
                min_lines: 3,
                max_depth: 3,
                priority: 9,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^public .*\(.*\)".to_string(),
                r"^private .*\(.*\)".to_string(),
                r"^protected .*\(.*\)".to_string(),
                r"^class ".to_string(),
                r"^interface ".to_string(),
            ])
            .with_chunk_size(15);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(JavaProcessor);

// Go processor
pub struct GoProcessor {
    processor: BaseProcessor,
}

impl Default for GoProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl GoProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_go::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "function_declaration".to_string(),
                    "method_declaration".to_string(),
                    "type_declaration".to_string(),
                    "struct_type".to_string(),
                ],
                min_length: 35,
                min_lines: 2,
                max_depth: 3,
                priority: 8,
                include_context: false,
            }])
            .with_fallback_patterns(vec![r"^func ".to_string(), r"^type ".to_string()])
            .with_chunk_size(15);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(GoProcessor);

// C processor
pub struct CProcessor {
    processor: BaseProcessor,
}

impl Default for CProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_c::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "function_definition".to_string(),
                    "struct_specifier".to_string(),
                    "enum_specifier".to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 2,
                priority: 8,
                include_context: false,
            }])
            .with_fallback_patterns(vec![
                r"^[a-zA-Z_][a-zA-Z0-9_]*\s*\(".to_string(),
                r"^struct ".to_string(),
                r"^enum ".to_string(),
                r"^typedef ".to_string(),
            ])
            .with_chunk_size(15);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(CProcessor);

// C++ processor
pub struct CppProcessor {
    processor: BaseProcessor,
}

impl Default for CppProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CppProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_cpp::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "function_definition".to_string(),
                    "class_specifier".to_string(),
                    "struct_specifier".to_string(),
                    "template_declaration".to_string(),
                ],
                min_length: 40,
                min_lines: 3,
                max_depth: 3,
                priority: 9,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^template ".to_string(),
                r"^class ".to_string(),
                r"^struct ".to_string(),
                r"^[a-zA-Z_][a-zA-Z0-9_]*\s*\(".to_string(),
            ])
            .with_chunk_size(15);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(CppProcessor);

// C# processor
pub struct CSharpProcessor {
    processor: BaseProcessor,
}

impl Default for CSharpProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CSharpProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_c_sharp::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "method_declaration".to_string(),
                    "class_declaration".to_string(),
                    "interface_declaration".to_string(),
                    "property_declaration".to_string(),
                ],
                min_length: 40,
                min_lines: 3,
                max_depth: 3,
                priority: 9,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^public .*\(.*\)".to_string(),
                r"^private .*\(.*\)".to_string(),
                r"^protected .*\(.*\)".to_string(),
                r"^class ".to_string(),
                r"^interface ".to_string(),
            ])
            .with_chunk_size(15);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(CSharpProcessor);
