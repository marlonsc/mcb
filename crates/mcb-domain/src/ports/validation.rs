use std::fmt::Display;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Opaque error type for validator failures.
pub type ValidatorError = Box<dyn std::error::Error + Send + Sync>;
/// Specialized result for validation operations.
pub type ValidatorResult<T> = std::result::Result<T, ValidatorError>;

/// Severity level of a code violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    /// Blocking issue that must be fixed.
    Error,
    /// Non-blocking issue that should be fixed.
    Warning,
    /// Suggestion for improvement or informational note.
    Info,
}

impl Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "ERROR"),
            Self::Warning => write!(f, "WARNING"),
            Self::Info => write!(f, "INFO"),
        }
    }
}

/// Categories of code violations identified by validators.
///
/// Used for grouping and filtering results in the validation report.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    strum_macros::Display,
    strum_macros::EnumString,
)]
#[strum(ascii_case_insensitive)]
pub enum ViolationCategory {
    /// Architectural integrity, layering, and boundary violations.
    #[strum(serialize = "Architecture", serialize = "clean-architecture")]
    Architecture,
    /// General code quality and maintainability issues.
    #[strum(serialize = "Quality")]
    Quality,
    /// Project organization and module structure.
    #[strum(serialize = "Organization")]
    Organization,
    /// SOLID principles violations.
    #[strum(serialize = "SOLID")]
    Solid,
    /// Dependency Injection (linkme) registration and wiring issues.
    #[strum(
        serialize = "DI/linkme",
        serialize = "di",
        serialize = "dependency_injection"
    )]
    DependencyInjection,
    /// Configuration management and environment setup.
    #[strum(serialize = "Configuration")]
    Configuration,
    /// Web framework (Loco/Axum) specific patterns.
    #[strum(
        serialize = "Web Framework",
        serialize = "web-framework",
        serialize = "web_framework"
    )]
    WebFramework,
    /// Performance bottlenecks and inefficient patterns.
    #[strum(serialize = "Performance")]
    Performance,
    /// Async/await usage and potential deadlocks.
    #[strum(serialize = "Async")]
    Async,
    /// Missing or insufficient documentation (KISS).
    #[strum(serialize = "Documentation")]
    Documentation,
    /// Testing standards and coverage.
    #[strum(serialize = "Testing")]
    Testing,
    /// Naming conventions and style guide.
    #[strum(serialize = "Naming")]
    Naming,
    /// KISS (Keep It Simple, Stupid) principle violations.
    #[strum(serialize = "KISS")]
    Kiss,
    /// Refactoring opportunities and legacy code smells.
    #[strum(serialize = "Refactoring", serialize = "migration")]
    Refactoring,
    /// Error boundary and graceful degradation.
    #[strum(serialize = "Error Boundary", serialize = "error_boundary")]
    ErrorBoundary,
    /// Implementation details and best practices.
    #[strum(serialize = "Implementation")]
    Implementation,
    /// PMAT (Persistence, Models, Actions, Tasks) pattern.
    #[strum(serialize = "PMAT")]
    Pmat,
    /// Metrics and thresholds.
    #[strum(serialize = "Metrics")]
    Metrics,
}

/// Shared interface for all code violations.
pub trait Violation: Display + Send + Sync + std::fmt::Debug {
    /// Unique identifier for the violation type (e.g., "ARCH001").
    fn id(&self) -> &str;
    /// Category used for grouping violations in reports.
    fn category(&self) -> ViolationCategory;
    /// Importance of the violation.
    fn severity(&self) -> Severity;
    /// Path to the file containing the violation, if applicable.
    fn file(&self) -> Option<&PathBuf>;
    /// Line number where the violation occurs, if applicable.
    fn line(&self) -> Option<usize>;

    /// Human-readable description of the violation.
    fn message(&self) -> String {
        self.to_string()
    }

    /// Actionable advice on how to fix the violation.
    fn suggestion(&self) -> Option<String> {
        None
    }

    /// Helper to convert the violation into a trait object.
    fn boxed(self) -> Box<dyn Violation>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

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

/// Configuration for a validation run.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// The absolute path to the workspace root.
    pub workspace_root: PathBuf,
    /// Additional source paths to include in the scan.
    pub additional_src_paths: Vec<PathBuf>,
    /// Glob patterns for excluding files or directories.
    pub exclude_patterns: Vec<String>,
}

impl ValidationConfig {
    /// Create a new validation configuration for the specified workspace.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let raw: PathBuf = workspace_root.into();
        let canonical = std::fs::canonicalize(&raw).unwrap_or(raw);
        Self {
            workspace_root: canonical,
            additional_src_paths: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }

    /// Add an extra directory to include in the validation scan.
    #[must_use]
    pub fn with_additional_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.additional_src_paths.push(path.into());
        self
    }

    /// Add a glob pattern for excluding files or directories from the validation scan.
    #[must_use]
    pub fn with_exclude_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.exclude_patterns.push(pattern.into());
        self
    }

    /// Check if a specific path should be excluded according to the configured patterns.
    #[must_use]
    pub fn should_exclude(&self, path: &Path) -> bool {
        let Some(path_str) = path.to_str() else {
            return false;
        };
        self.exclude_patterns
            .iter()
            .any(|pattern| path_str.contains(pattern))
    }
}

/// Function type for a validation check.
pub type CheckFn<'a> = Box<dyn FnOnce() -> ValidatorResult<Vec<Box<dyn Violation>>> + 'a>;

/// A named validation check entry.
pub struct NamedCheck<'a> {
    /// The human-readable name of the check.
    pub name: &'static str,
    /// The logic of the check.
    pub run: CheckFn<'a>,
}

impl<'a> NamedCheck<'a> {
    /// Create a new named check.
    pub fn new(
        name: &'static str,
        run: impl FnOnce() -> ValidatorResult<Vec<Box<dyn Violation>>> + 'a,
    ) -> Self {
        Self {
            name,
            run: Box::new(run),
        }
    }
}

/// Runs a series of named checks and returns the combined violations.
///
/// # Errors
/// Returns a `ValidatorError` if any individual check fails or returns an error.
pub fn run_checks(
    validator_name: &str,
    checks: Vec<NamedCheck<'_>>,
) -> ValidatorResult<Vec<Box<dyn Violation>>> {
    let mut violations = Vec::new();
    for check in checks {
        let t = std::time::Instant::now();
        let v = (check.run)()?;
        crate::debug!(
            validator_name,
            &format!("{} done", check.name),
            &format!("violations={} elapsed={:.2?}", v.len(), t.elapsed())
        );
        violations.extend(v);
    }
    Ok(violations)
}

/// Interface for implementing codebase validation rules.
///
/// Validators are responsible for checking certain aspects of the code (naming, SOLID, cleanliness)
/// and returning a list of violations found.
pub trait Validator: Send + Sync {
    /// Returns the unique name of this validator.
    fn name(&self) -> &'static str;

    /// Returns a list of individual named checks for this validator.
    ///
    /// # Errors
    /// Returns a `ValidatorResult` which may contain an error if check preparation fails.
    fn checks<'a>(&'a self, _config: &'a ValidationConfig) -> ValidatorResult<Vec<NamedCheck<'a>>> {
        Ok(Vec::new())
    }

    /// Performs the full validation scan for this validator.
    ///
    /// # Errors
    /// Returns a `ValidatorResult` error if the validation process fails.
    fn validate(&self, config: &ValidationConfig) -> ValidatorResult<Vec<Box<dyn Violation>>> {
        run_checks(self.name(), self.checks(config)?)
    }

    /// Check if this validator should run by default in a standard scan.
    fn enabled_by_default(&self) -> bool {
        true
    }

    /// Get a human-readable description of what this validator checks.
    fn description(&self) -> &'static str {
        ""
    }

    /// Get the list of languages supported by this validator.
    fn supported_languages(&self) -> &[LanguageId] {
        &[LanguageId::Rust]
    }
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
