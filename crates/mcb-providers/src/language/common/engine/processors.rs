//!
//! **Documentation**: [docs/modules/providers.md](../../../../../docs/modules/providers.md)
//!
//! Language processor registry
//!
//! Provides the static registry of language-specific processors.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::language::{
    CProcessor, CSharpProcessor, CppProcessor, GoProcessor, JavaProcessor, JavaScriptProcessor,
    KotlinProcessor, LanguageProcessor, PhpProcessor, PythonProcessor, RubyProcessor,
    RustProcessor, SwiftProcessor,
};

/// Language processor registry
pub(crate) static LANGUAGE_PROCESSORS: LazyLock<
    HashMap<String, Box<dyn LanguageProcessor + Send + Sync>>,
> = LazyLock::new(|| {
    let mut processors: HashMap<String, Box<dyn LanguageProcessor + Send + Sync>> = HashMap::new();

    processors.insert(
        "rust".to_owned(),
        Box::new(RustProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "python".to_owned(),
        Box::new(PythonProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "javascript".to_owned(),
        Box::new(JavaScriptProcessor::new(false)) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "typescript".to_owned(),
        Box::new(JavaScriptProcessor::new(true)) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "go".to_owned(),
        Box::new(GoProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "java".to_owned(),
        Box::new(JavaProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "c".to_owned(),
        Box::new(CProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "cpp".to_owned(),
        Box::new(CppProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "csharp".to_owned(),
        Box::new(CSharpProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "ruby".to_owned(),
        Box::new(RubyProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "php".to_owned(),
        Box::new(PhpProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "swift".to_owned(),
        Box::new(SwiftProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "kotlin".to_owned(),
        Box::new(KotlinProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );

    processors
});
