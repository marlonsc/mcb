use std::path::Path;

use rust_code_analysis::{LANG, guess_language};

pub use mcb_domain::ports::validation::LanguageId;

pub fn language_to_rca(lang: LanguageId) -> LANG {
    match lang {
        LanguageId::Rust => LANG::Rust,
        LanguageId::Python => LANG::Python,
        LanguageId::JavaScript => LANG::Mozjs,
        LanguageId::TypeScript => LANG::Typescript,
        LanguageId::Java => LANG::Java,
        LanguageId::Cpp => LANG::Cpp,
        LanguageId::Kotlin => LANG::Kotlin,
        LanguageId::Go
        | LanguageId::Ruby
        | LanguageId::Shell
        | LanguageId::Yaml
        | LanguageId::Toml
        | LanguageId::Json
        | LanguageId::Markdown
        | LanguageId::Html
        | LanguageId::Css
        | LanguageId::Sql
        | LanguageId::Dockerfile
        | LanguageId::Makefile
        | LanguageId::Protobuf => LANG::Preproc,
    }
}

pub fn language_from_rca(lang: LANG) -> Option<LanguageId> {
    match lang {
        LANG::Rust => Some(LanguageId::Rust),
        LANG::Python => Some(LanguageId::Python),
        LANG::Mozjs | LANG::Javascript => Some(LanguageId::JavaScript),
        LANG::Typescript | LANG::Tsx => Some(LanguageId::TypeScript),
        LANG::Java => Some(LanguageId::Java),
        LANG::Cpp => Some(LanguageId::Cpp),
        LANG::Kotlin => Some(LanguageId::Kotlin),
        LANG::Ccomment | LANG::Preproc => None,
    }
}

pub struct LanguageDetector {
    _private: (),
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageDetector {
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    pub fn detect(&self, path: &Path, content: Option<&str>) -> Option<LanguageId> {
        let source = content.map_or_else(
            || std::fs::read(path).unwrap_or_default(),
            |c| c.as_bytes().to_vec(),
        );
        let (rca_lang, _) = guess_language(&source, path);
        if let Some(lang) = rca_lang.and_then(language_from_rca) {
            return Some(lang);
        }

        if let Some(ext) = path.extension().and_then(|ext| ext.to_str())
            && let Some(lang) = LanguageId::from_extension(ext)
        {
            return Some(lang);
        }

        if let Some(filename) = path.file_name().and_then(|name| name.to_str())
            && let Some(lang) = LanguageId::from_filename(filename)
        {
            return Some(lang);
        }

        let first_line = content.and_then(|body| body.lines().next()).or_else(|| {
            std::str::from_utf8(&source)
                .ok()
                .and_then(|body| body.lines().next())
        });

        first_line.and_then(LanguageId::from_shebang)
    }

    #[must_use]
    pub fn detect_name(&self, path: &Path, content: Option<&str>) -> Option<String> {
        self.detect(path, content).map(|id| id.name().to_owned())
    }

    #[must_use]
    pub fn detect_rca_lang(&self, path: &Path, content: Option<&str>) -> Option<LANG> {
        let source = content.map_or_else(
            || std::fs::read(path).unwrap_or_default(),
            |c| c.as_bytes().to_vec(),
        );
        let (rca_lang, _) = guess_language(&source, path);
        rca_lang.and_then(|lang| language_from_rca(lang).map(|_| lang))
    }

    #[must_use]
    pub fn supported_language_names(&self) -> Vec<String> {
        vec![
            "rust".to_owned(),
            "python".to_owned(),
            "javascript".to_owned(),
            "typescript".to_owned(),
            "go".to_owned(),
            "java".to_owned(),
            "cpp".to_owned(),
            "kotlin".to_owned(),
            "ruby".to_owned(),
            "shell".to_owned(),
            "yaml".to_owned(),
            "toml".to_owned(),
            "json".to_owned(),
            "markdown".to_owned(),
            "html".to_owned(),
            "css".to_owned(),
            "sql".to_owned(),
            "dockerfile".to_owned(),
            "makefile".to_owned(),
            "protobuf".to_owned(),
        ]
    }

    #[must_use]
    pub fn matches_languages(
        &self,
        path: &Path,
        content: Option<&str>,
        allowed_languages: &[String],
    ) -> bool {
        self.detect_name(path, content)
            .is_some_and(|language| allowed_languages.contains(&language))
    }
}
