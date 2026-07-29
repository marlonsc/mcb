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
> = LazyLock::new(build_processors);

type BoxedProcessor = Box<dyn LanguageProcessor + Send + Sync>;

fn build_processors() -> HashMap<String, BoxedProcessor> {
    let entries: [(&str, BoxedProcessor); 13] = [
        ("rust", Box::new(RustProcessor::new())),
        ("python", Box::new(PythonProcessor::new())),
        ("javascript", Box::new(JavaScriptProcessor::new(false))),
        ("typescript", Box::new(JavaScriptProcessor::new(true))),
        ("go", Box::new(GoProcessor::new())),
        ("java", Box::new(JavaProcessor::new())),
        ("c", Box::new(CProcessor::new())),
        ("cpp", Box::new(CppProcessor::new())),
        ("csharp", Box::new(CSharpProcessor::new())),
        ("ruby", Box::new(RubyProcessor::new())),
        ("php", Box::new(PhpProcessor::new())),
        ("swift", Box::new(SwiftProcessor::new())),
        ("kotlin", Box::new(KotlinProcessor::new())),
    ];
    entries
        .into_iter()
        .map(|(name, processor)| (name.to_owned(), processor))
        .collect()
}
