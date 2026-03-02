use std::fmt::Display;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub type ValidatorError = Box<dyn std::error::Error + Send + Sync>;
pub type ValidatorResult<T> = std::result::Result<T, ValidatorError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Error,
    Warning,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum ViolationCategory {
    /// Architectural integrity, layering, and boundary violations.
    Architecture,
    /// General code quality and maintainability issues.
    Quality,
    /// Project organization and module structure.
    Organization,
    /// SOLID principles violations.
    Solid,
    /// Dependency Injection (linkme) registration and wiring issues.
    DependencyInjection,
    /// Configuration management and environment setup.
    Configuration,
    /// Web framework (Loco/Axum) specific patterns.
    WebFramework,
    /// Performance bottlenecks and inefficient patterns.
    Performance,
    /// Async/await usage and potential deadlocks.
    Async,
    /// Missing or insufficient documentation (KISS).
    Documentation,
    /// Testing standards and coverage.
    Testing,
    /// Naming conventions and style guide.
    Naming,
    /// KISS (Keep It Simple, Stupid) principle violations.
    Kiss,
    /// Refactoring opportunities and legacy code smells.
    Refactoring,
    /// Error boundary and graceful degradation.
    ErrorBoundary,
    /// Implementation details and best practices.
    Implementation,
    /// PMAT (Persistence, Models, Actions, Tasks) pattern.
    Pmat,
}

impl Display for ViolationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Architecture => write!(f, "Architecture"),
            Self::Quality => write!(f, "Quality"),
            Self::Organization => write!(f, "Organization"),
            Self::Solid => write!(f, "SOLID"),
            Self::DependencyInjection => write!(f, "DI/linkme"),
            Self::Configuration => write!(f, "Configuration"),
            Self::WebFramework => write!(f, "Web Framework"),
            Self::Performance => write!(f, "Performance"),
            Self::Async => write!(f, "Async"),
            Self::Documentation => write!(f, "Documentation"),
            Self::Testing => write!(f, "Testing"),
            Self::Naming => write!(f, "Naming"),
            Self::Kiss => write!(f, "KISS"),
            Self::Refactoring => write!(f, "Refactoring"),
            Self::ErrorBoundary => write!(f, "Error Boundary"),
            Self::Implementation => write!(f, "Implementation"),
            Self::Pmat => write!(f, "PMAT"),
        }
    }
}

pub trait Violation: Display + Send + Sync + std::fmt::Debug {
    fn id(&self) -> &str;
    fn category(&self) -> ViolationCategory;
    fn severity(&self) -> Severity;
    fn file(&self) -> Option<&PathBuf>;
    fn line(&self) -> Option<usize>;

    fn message(&self) -> String {
        self.to_string()
    }

    fn suggestion(&self) -> Option<String> {
        None
    }

    fn boxed(self) -> Box<dyn Violation>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageId {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Java,
    Cpp,
    Kotlin,
    Go,
    Ruby,
    Shell,
    Yaml,
    Toml,
    Json,
    Markdown,
    Html,
    Css,
    Sql,
    Dockerfile,
    Makefile,
    Protobuf,
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
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let raw: PathBuf = workspace_root.into();
        let canonical = std::fs::canonicalize(&raw).unwrap_or(raw);
        Self {
            workspace_root: canonical,
            additional_src_paths: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_additional_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.additional_src_paths.push(path.into());
        self
    }

    #[must_use]
    pub fn with_exclude_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.exclude_patterns.push(pattern.into());
        self
    }

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

pub type CheckFn<'a> = Box<dyn FnOnce() -> ValidatorResult<Vec<Box<dyn Violation>>> + 'a>;

pub struct NamedCheck<'a> {
    pub name: &'static str,
    pub run: CheckFn<'a>,
}

impl<'a> NamedCheck<'a> {
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

pub trait Validator: Send + Sync {
    fn name(&self) -> &'static str;

    fn checks<'a>(&'a self, _config: &'a ValidationConfig) -> ValidatorResult<Vec<NamedCheck<'a>>> {
        Ok(Vec::new())
    }

    fn validate(&self, config: &ValidationConfig) -> ValidatorResult<Vec<Box<dyn Violation>>> {
        run_checks(self.name(), self.checks(config)?)
    }

    fn enabled_by_default(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        ""
    }

    fn supported_languages(&self) -> &[LanguageId] {
        &[LanguageId::Rust]
    }
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
    ];

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

    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.to_ascii_lowercase();
        Self::NAME_EQUIVALENTS
            .iter()
            .find_map(|(n, lang)| (*n == lower).then_some(*lang))
    }

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
