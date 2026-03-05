//! Supported programming languages for scanners and validators.

use serde::{Deserialize, Serialize};

/// Supported programming languages for scanners and validators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageId {
    /// Rust source code.
    Rust,
    /// Python source code.
    Python,
    /// JavaScript source code.
    JavaScript,
    /// TypeScript source code.
    TypeScript,
    /// TypeScript with JSX.
    Tsx,
    /// Java source code.
    Java,
    /// C source code.
    C,
    /// C/C++ source code.
    Cpp,
    /// Kotlin source code.
    Kotlin,
    /// Go source code.
    Go,
    /// Ruby source code.
    Ruby,
    /// Shell scripts (Bash, Sh, etc.).
    Shell,
    /// YAML configuration files.
    Yaml,
    /// TOML configuration files.
    Toml,
    /// JSON data files.
    Json,
    /// Markdown documentation.
    Markdown,
    /// HTML templates or pages.
    Html,
    /// CSS stylesheets.
    Css,
    /// SQL database scripts.
    Sql,
    /// Docker configuration files.
    Dockerfile,
    /// Build system Makefiles.
    Makefile,
    /// Protocol Buffer definitions.
    Protobuf,
    /// PHP source code.
    Php,
    /// Swift source code.
    Swift,
}

struct LanguageData {
    id: LanguageId,
    name: &'static str,
    extensions: &'static [&'static str],
    filenames: &'static [&'static str],
}

impl LanguageId {
    const NAME_EQUIVALENTS: &'static [(&'static str, Self)] = &[
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
        ("php", Self::Php),
        ("swift", Self::Swift),
        ("tsx", Self::Tsx),
        ("c", Self::C),
    ];

    const LANGUAGE_DATA: &'static [LanguageData] = &[
        LanguageData {
            id: Self::Rust,
            name: "rust",
            extensions: &["rs"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Python,
            name: "python",
            extensions: &["py", "pyi", "pyw"],
            filenames: &[],
        },
        LanguageData {
            id: Self::JavaScript,
            name: "javascript",
            extensions: &["js", "mjs", "cjs", "jsx"],
            filenames: &[],
        },
        LanguageData {
            id: Self::TypeScript,
            name: "typescript",
            extensions: &["ts", "mts", "cts", "tsx"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Tsx,
            name: "tsx",
            extensions: &["tsx"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Java,
            name: "java",
            extensions: &["java"],
            filenames: &[],
        },
        LanguageData {
            id: Self::C,
            name: "c",
            extensions: &["c", "h"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Cpp,
            name: "cpp",
            extensions: &["cpp", "cc", "cxx", "hpp", "hxx", "mm", "m"],
            filenames: &["cmakelists.txt"],
        },
        LanguageData {
            id: Self::Kotlin,
            name: "kotlin",
            extensions: &["kt", "kts"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Go,
            name: "go",
            extensions: &["go"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Ruby,
            name: "ruby",
            extensions: &["rb", "rake", "gemspec"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Shell,
            name: "shell",
            extensions: &["sh", "bash", "zsh", "ksh"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Yaml,
            name: "yaml",
            extensions: &["yaml", "yml"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Toml,
            name: "toml",
            extensions: &["toml"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Json,
            name: "json",
            extensions: &["json", "jsonc"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Markdown,
            name: "markdown",
            extensions: &["md", "markdown", "mdown", "mkd"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Html,
            name: "html",
            extensions: &["html", "htm", "xhtml"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Css,
            name: "css",
            extensions: &["css", "scss", "sass", "less"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Sql,
            name: "sql",
            extensions: &["sql"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Dockerfile,
            name: "dockerfile",
            extensions: &["dockerfile"],
            filenames: &["dockerfile"],
        },
        LanguageData {
            id: Self::Makefile,
            name: "makefile",
            extensions: &["makefile", "mk"],
            filenames: &["makefile", "gnumakefile"],
        },
        LanguageData {
            id: Self::Protobuf,
            name: "protobuf",
            extensions: &["proto"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Php,
            name: "php",
            extensions: &["php", "php4", "php5", "phtml"],
            filenames: &[],
        },
        LanguageData {
            id: Self::Swift,
            name: "swift",
            extensions: &["swift"],
            filenames: &[],
        },
    ];
    /// Get the primary lowercase name of the language.
    #[must_use]
    pub fn name(&self) -> &'static str {
        Self::LANGUAGE_DATA
            .iter()
            .find(|d| d.id == *self)
            .map_or("unknown", |d| d.name)
    }

    /// Get typical file extensions associated with this language.
    #[must_use]
    pub fn extensions(&self) -> &'static [&'static str] {
        Self::LANGUAGE_DATA
            .iter()
            .find(|d| d.id == *self)
            .map_or(&[], |d| d.extensions)
    }

    /// Resolve a language ID from its human-readable name or alias.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.to_ascii_lowercase();
        Self::NAME_EQUIVALENTS
            .iter()
            .find_map(|(n, lang)| (*n == lower).then_some(*lang))
    }

    /// Infer the language from a file extension.
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        let normalized = ext.trim().trim_start_matches('.').to_ascii_lowercase();
        Self::LANGUAGE_DATA
            .iter()
            .find(|d| d.extensions.contains(&normalized.as_str()))
            .map(|d| d.id)
    }

    /// Infer the language from a specific filename (e.g., "Dockerfile").
    #[must_use]
    pub fn from_filename(filename: &str) -> Option<Self> {
        let lower = filename.to_ascii_lowercase();
        Self::LANGUAGE_DATA
            .iter()
            .find(|d| d.filenames.contains(&lower.as_str()))
            .map(|d| d.id)
    }

    /// Identify the language by checking the file shebang line.
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

impl std::str::FromStr for LanguageId {
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
