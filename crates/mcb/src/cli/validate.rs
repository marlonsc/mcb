//! Validate command - runs architecture validation
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::path::PathBuf;
use std::time::Instant;

use clap::Args;

/// Arguments for the validate command
#[derive(Args, Debug, Clone)]
pub struct ValidateArgs {
    /// Path to workspace root (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Quick mode (summary only, no file details)
    #[arg(long)]
    pub quick: bool,

    /// Strict mode (fail on warnings, exit code 1)
    #[arg(long)]
    pub strict: bool,

    /// Custom rules directory
    #[arg(long)]
    pub rules: Option<PathBuf>,

    /// Specific validators to run (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub validators: Option<Vec<String>>,

    /// Minimum severity: error, warning, info
    #[arg(long, default_value = "warning")]
    pub severity: String,

    /// Output format: text, json
    #[arg(long, default_value = "text")]
    pub format: String,

    /// Silent mode: suppress all progress output on stderr
    #[arg(long, short = 's')]
    pub silent: bool,

    /// Debug mode: show detailed validator internals
    #[arg(long, short = 'd')]
    pub debug: bool,

    /// Trace mode: show everything including per-file processing
    #[arg(long, short = 't')]
    pub trace: bool,
}

/// Validation result for exit code determination
pub struct ValidationResult {
    /// Number of error violations found
    pub errors: usize,
    /// Number of warning violations found
    pub warnings: usize,
    /// Number of info violations found
    pub _infos: usize,
    /// Whether strict mode was enabled
    pub strict_mode: bool,
}

impl ValidationResult {
    /// Returns true if validation failed based on mode
    #[must_use]
    pub fn failed(&self) -> bool {
        if self.strict_mode {
            self.errors > 0 || self.warnings > 0
        } else {
            self.errors > 0
        }
    }
}

impl ValidateArgs {
    /// Initialize logging based on verbosity flags
    fn init_logging(&self) {
        use mcb_domain::ports::LogLevel;

        if self.silent {
            // No logging at all — set_log_fn is never called, so dispatch is a no-op
            return;
        }

        let level = if self.trace {
            LogLevel::Trace
        } else if self.debug {
            LogLevel::Debug
        } else {
            LogLevel::Info
        };

        mcb_infrastructure::logging::set_stderr_log_level(level);
        mcb_domain::infra::logging::set_log_fn(mcb_infrastructure::logging::stderr_log_fn);
    }

    /// Print a progress message to stderr (respects --silent)
    #[allow(clippy::print_stderr)]
    fn progress(&self, msg: &str) {
        if !self.silent {
            eprintln!("{msg}");
        }
    }

    /// Execute the validate command
    /// # Errors
    /// Returns an error if validation setup or execution fails.
    pub fn execute(self) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        use mcb_validate::{GenericReporter, ValidationConfig, ValidatorRegistry};

        self.init_logging();

        // Resolve workspace root
        let workspace_root = if self.path.is_absolute() {
            self.path.clone()
        } else {
            std::env::current_dir()?.join(&self.path)
        };

        self.progress(&format!(
            "● Validating workspace: {}",
            workspace_root.display()
        ));

        // Build validation config
        let config = ValidationConfig::new(&workspace_root);

        let registry = ValidatorRegistry::standard_for(&workspace_root);

        let validator_count = if let Some(ref v) = self.validators {
            v.len()
        } else {
            registry.validators().len()
        };

        self.progress(&format!("● Running {validator_count} validator(s)..."));

        let started = Instant::now();

        // Run validation
        let report = if let Some(ref validators) = self.validators {
            let validator_names: Vec<&str> = validators.iter().map(String::as_str).collect();
            let violations = registry.validate_named(&config, &validator_names)?;
            GenericReporter::create_report(&violations, workspace_root.clone())
        } else {
            let violations = registry.validate_all(&config)?;
            GenericReporter::create_report(&violations, workspace_root.clone())
        };

        let elapsed = started.elapsed();

        self.progress(&format!("● Done in {elapsed:.2?}"));

        // Format output
        match self.format.as_str() {
            "json" => {
                Self::print_json(&report)?;
            }
            _ => {
                self.print_text(&report);
            }
        }

        // Return counts for exit code
        Ok(ValidationResult {
            errors: report.summary.errors,
            warnings: report.summary.warnings,
            _infos: report.summary.infos,
            strict_mode: self.strict,
        })
    }

    /// Print report as JSON
    fn print_json(report: &mcb_validate::GenericReport) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(report)?;
        println!("{json}");
        Ok(())
    }

    /// Print report as text
    fn print_text(&self, report: &mcb_validate::GenericReport) {
        let severity_threshold = self.get_severity_threshold();

        // Print violations (unless quick mode)
        if !self.quick {
            Self::print_violations(report, severity_threshold);
        }

        // Print summary
        self.print_summary(report);
    }

    fn get_severity_threshold(&self) -> u8 {
        match self.severity.as_str() {
            "error" => 0,   // Only errors
            "warning" => 1, // Errors + warnings
            _ => 2,         // All (including info)
        }
    }

    fn print_violations(report: &mcb_validate::GenericReport, threshold: u8) {
        let mut has_violations = false;

        for violations in report.violations_by_category.values() {
            for violation in violations {
                if Self::should_print_violation(violation, threshold) {
                    has_violations = true;
                    Self::print_single_violation(violation);
                }
            }
        }

        if has_violations {
            println!();
        }
    }

    fn should_print_violation(violation: &mcb_validate::ViolationEntry, threshold: u8) -> bool {
        let sev_level = match violation.severity.as_str() {
            "ERROR" => 0,
            "WARNING" => 1,
            _ => 2,
        };
        sev_level <= threshold
    }

    fn print_single_violation(violation: &mcb_validate::ViolationEntry) {
        let file_display = violation.file.as_deref().unwrap_or("-");
        let line = violation.line.unwrap_or(0);

        println!(
            "[{}] {}: {} ({}:{})",
            violation.severity, violation.id, violation.message, file_display, line
        );
        if let Some(ref suggestion) = violation.suggestion {
            println!("  → {suggestion}");
        }
    }

    fn print_summary(&self, report: &mcb_validate::GenericReport) {
        println!(
            "Validation complete: {} error(s), {} warning(s), {} info(s)",
            report.summary.errors, report.summary.warnings, report.summary.infos
        );

        // Print category breakdown (unless quick mode)
        if !self.quick && !report.summary.by_category.is_empty() {
            println!("\nBy category:");
            for (category, count) in &report.summary.by_category {
                println!("  {category}: {count}");
            }
        }
    }
}
