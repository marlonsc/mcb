//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Language Detection for Rule Filtering
//!
//! Detects programming language from file extensions and content patterns.
//! Uses `rust-code-analysis` directly for language detection.
//!
//! Previously delegated to the `mcb-language-support` crate; all logic is now
//! inlined here to remove that dependency.

use std::path::Path;
use std::str::FromStr;

use rust_code_analysis::{LANG, guess_language};

/// Supported programming languages.
///
/// Maps to `rust_code_analysis::LANG` while providing a stable,
/// serializable representation. Only languages that RCA actually
/// supports are included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageId {
    /// Rust
    Rust,
    /// Python
    Python,
    /// JavaScript (including JSX)
    JavaScript,
    /// TypeScript (including TSX)
    TypeScript,
    /// Java
    Java,
    /// C / C++ (RCA treats them identically)
    Cpp,
    /// Kotlin
    Kotlin,
    /// Go
    Go,
    /// Ruby
    Ruby,
    /// Shell (bash, zsh, ksh)
    Shell,
    /// YAML
    Yaml,
    /// TOML
    Toml,
    /// JSON / JSONC
    Json,
    /// Markdown
    Markdown,
    /// HTML / XHTML
    Html,
    /// CSS / SCSS / SASS / Less
    Css,
    /// SQL
    Sql,
    /// Dockerfile
    Dockerfile,
    /// Makefile
    Makefile,
    /// Protocol Buffers
    Protobuf,
}

impl LanguageId {
    const NAME_EQUIVALENTS: &'static [(&'static str, LanguageId)] = &[
        ("rust", Self::Rust),
        ("rs", Self::Rust),
        ("python", Self::Python),
        ("py", Self::Python),
        ("javascript", Self::JavaScript),
        ("js", Self::JavaScript),
        ("jsx", Self::JavaScript),
        ("mozjs", Self::JavaScript),
        ("typescript", Self::TypeScript),
        ("ts", Self::TypeScript),
        ("tsx", Self::TypeScript),
        ("java", Self::Java),
        ("cpp", Self::Cpp),
        ("c++", Self::Cpp),
        ("c", Self::Cpp),
        ("cxx", Self::Cpp),
        ("kotlin", Self::Kotlin),
        ("kt", Self::Kotlin),
        ("go", Self::Go),
        ("golang", Self::Go),
        ("ruby", Self::Ruby),
        ("rb", Self::Ruby),
        ("shell", Self::Shell),
        ("sh", Self::Shell),
        ("bash", Self::Shell),
        ("zsh", Self::Shell),
        ("ksh", Self::Shell),
        ("yaml", Self::Yaml),
        ("yml", Self::Yaml),
        ("toml", Self::Toml),
        ("json", Self::Json),
        ("jsonc", Self::Json),
        ("markdown", Self::Markdown),
        ("md", Self::Markdown),
        ("html", Self::Html),
        ("htm", Self::Html),
        ("css", Self::Css),
        ("scss", Self::Css),
        ("sass", Self::Css),
        ("less", Self::Css),
        ("sql", Self::Sql),
        ("dockerfile", Self::Dockerfile),
        ("docker", Self::Dockerfile),
        ("makefile", Self::Makefile),
        ("make", Self::Makefile),
        ("gnumake", Self::Makefile),
        ("protobuf", Self::Protobuf),
        ("proto", Self::Protobuf),
    ];

    /// Get the canonical name of the language.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Java => "java",
            Self::Cpp => "cpp",
            Self::Kotlin => "kotlin",
            Self::Go => "go",
            Self::Ruby => "ruby",
            Self::Shell => "shell",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Json => "json",
            Self::Markdown => "markdown",
            Self::Html => "html",
            Self::Css => "css",
            Self::Sql => "sql",
            Self::Dockerfile => "dockerfile",
            Self::Makefile => "makefile",
            Self::Protobuf => "protobuf",
        }
    }

    /// Convert to `rust_code_analysis::LANG`.
    #[must_use]
    pub fn to_rca_lang(&self) -> LANG {
        match self {
            Self::Rust => LANG::Rust,
            Self::Python => LANG::Python,
            Self::JavaScript => LANG::Mozjs,
            Self::TypeScript => LANG::Typescript,
            Self::Java => LANG::Java,
            Self::Cpp => LANG::Cpp,
            Self::Kotlin => LANG::Kotlin,
            Self::Go
            | Self::Ruby
            | Self::Shell
            | Self::Yaml
            | Self::Toml
            | Self::Json
            | Self::Markdown
            | Self::Html
            | Self::Css
            | Self::Sql
            | Self::Dockerfile
            | Self::Makefile
            | Self::Protobuf => LANG::Preproc,
        }
    }

    /// Create from `rust_code_analysis::LANG`.
    #[must_use]
    pub fn from_rca_lang(lang: LANG) -> Option<Self> {
        match lang {
            LANG::Rust => Some(Self::Rust),
            LANG::Python => Some(Self::Python),
            LANG::Mozjs | LANG::Javascript => Some(Self::JavaScript),
            LANG::Typescript | LANG::Tsx => Some(Self::TypeScript),
            LANG::Java => Some(Self::Java),
            LANG::Cpp => Some(Self::Cpp),
            LANG::Kotlin => Some(Self::Kotlin),
            LANG::Ccomment | LANG::Preproc => None,
        }
    }

    /// Common file extensions for this language.
    #[must_use]
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Rust => &["rs"],
            Self::Python => &["py", "pyi", "pyw"],
            Self::JavaScript => &["js", "mjs", "cjs", "jsx"],
            Self::TypeScript => &["ts", "mts", "cts", "tsx"],
            Self::Java => &["java"],
            Self::Cpp => &["c", "h", "cpp", "cc", "cxx", "hpp", "hxx", "mm", "m"],
            Self::Kotlin => &["kt", "kts"],
            Self::Go => &["go"],
            Self::Ruby => &["rb", "rake", "gemspec"],
            Self::Shell => &["sh", "bash", "zsh", "ksh"],
            Self::Yaml => &["yaml", "yml"],
            Self::Toml => &["toml"],
            Self::Json => &["json", "jsonc"],
            Self::Markdown => &["md", "markdown", "mdown", "mkd"],
            Self::Html => &["html", "htm", "xhtml"],
            Self::Css => &["css", "scss", "sass", "less"],
            Self::Sql => &["sql"],
            Self::Dockerfile => &["dockerfile"],
            Self::Makefile => &["makefile", "mk"],
            Self::Protobuf => &["proto"],
        }
    }

    /// Try to create from a string name (case-insensitive).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.to_ascii_lowercase();
        Self::NAME_EQUIVALENTS
            .iter()
            .find_map(|(name, lang)| (*name == lower).then_some(*lang))
    }

    /// Resolve a `LanguageId` from a file extension (case-insensitive, leading dot stripped).
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        let normalized = ext.trim().trim_start_matches('.').to_ascii_lowercase();
        match normalized.as_str() {
            "rs" => Some(Self::Rust),
            "py" | "pyi" | "pyw" => Some(Self::Python),
            "js" | "mjs" | "cjs" | "jsx" => Some(Self::JavaScript),
            "ts" | "mts" | "cts" | "tsx" => Some(Self::TypeScript),
            "java" => Some(Self::Java),
            "c" | "h" | "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "mm" | "m" => Some(Self::Cpp),
            "kt" | "kts" => Some(Self::Kotlin),
            "go" => Some(Self::Go),
            "rb" | "rake" | "gemspec" => Some(Self::Ruby),
            "sh" | "bash" | "zsh" | "ksh" => Some(Self::Shell),
            "yaml" | "yml" => Some(Self::Yaml),
            "toml" => Some(Self::Toml),
            "json" | "jsonc" => Some(Self::Json),
            "md" | "markdown" | "mdown" | "mkd" => Some(Self::Markdown),
            "html" | "htm" | "xhtml" => Some(Self::Html),
            "css" | "scss" | "sass" | "less" => Some(Self::Css),
            "sql" => Some(Self::Sql),
            "proto" => Some(Self::Protobuf),
            "mk" => Some(Self::Makefile),
            _ => None,
        }
    }

    /// Resolve a `LanguageId` from a well-known filename (e.g. `Dockerfile`, `Makefile`).
    #[must_use]
    pub fn from_filename(filename: &str) -> Option<Self> {
        let lower = filename.to_ascii_lowercase();
        match lower.as_str() {
            "dockerfile" => Some(Self::Dockerfile),
            "makefile" | "gnumakefile" => Some(Self::Makefile),
            "cmakelists.txt" => Some(Self::Cpp),
            _ => None,
        }
    }

    /// Resolve a `LanguageId` from a shebang line (e.g. `#!/usr/bin/env python3`).
    #[must_use]
    pub fn from_shebang(first_line: &str) -> Option<Self> {
        let line = first_line.trim().to_ascii_lowercase();
        line.starts_with("#!")
            .then(|| {
                if line.contains("python") {
                    Self::Python
                } else if line.contains("ts-node") {
                    Self::TypeScript
                } else if line.contains("node") || line.contains("deno") {
                    Self::JavaScript
                } else if ["bash", "sh", "zsh", "ksh"]
                    .iter()
                    .any(|shell| line.contains(shell))
                {
                    Self::Shell
                } else {
                    Self::Ruby
                }
            })
            .filter(|lang| *lang != Self::Ruby || line.contains("ruby"))
    }
}

impl FromStr for LanguageId {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_name(s).ok_or_else(|| format!("Unknown language: {s}"))
    }
}

impl std::fmt::Display for LanguageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// Detects programming language from file paths using `rust_code_analysis::guess_language`.
pub struct LanguageDetector {
    _private: (),
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageDetector {
    /// Create a new language detector.
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Detect language from file path, returning a [`LanguageId`].
    pub fn detect(&self, path: &Path, content: Option<&str>) -> Option<LanguageId> {
        let source = content.map_or_else(
            // INTENTIONAL: File read for language detection; empty string skips file
            || std::fs::read(path).unwrap_or_default(),
            |c| c.as_bytes().to_vec(),
        );
        let (rca_lang, _) = guess_language(&source, path);
        if let Some(lang) = rca_lang.and_then(LanguageId::from_rca_lang) {
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

    /// Detect language and return as string name.
    #[must_use]
    pub fn detect_name(&self, path: &Path, content: Option<&str>) -> Option<String> {
        self.detect(path, content).map(|id| id.name().to_owned())
    }

    /// Detect language and return `LANG` enum for direct RCA usage.
    #[must_use]
    pub fn detect_rca_lang(&self, path: &Path, content: Option<&str>) -> Option<LANG> {
        let source = content.map_or_else(
            // INTENTIONAL: File read for language detection; empty string skips file
            || std::fs::read(path).unwrap_or_default(),
            |c| c.as_bytes().to_vec(),
        );
        let (rca_lang, _) = guess_language(&source, path);
        rca_lang.and_then(|lang| LanguageId::from_rca_lang(lang).map(|_| lang))
    }

    /// All supported language names.
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

    /// Check if a file matches any of the specified language names.
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
