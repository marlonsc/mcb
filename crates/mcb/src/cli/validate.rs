//! Validate command - runs architecture validation

use std::path::PathBuf;

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
}

/// Validation result for exit code determination
pub struct ValidationResult {
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
    pub strict_mode: bool,
}

impl ValidationResult {
    /// Returns true if validation failed based on mode
    pub fn failed(&self) -> bool {
        if self.strict_mode {
            self.errors > 0 || self.warnings > 0
        } else {
            self.errors > 0
        }
    }
}

impl ValidateArgs {
    /// Execute the validate command
    pub fn execute(self) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        use mcb_validate::{ArchitectureValidator, ValidationConfig};

        // Resolve workspace root
        let workspace_root = if self.path.is_absolute() {
            self.path.clone()
        } else {
            std::env::current_dir()?.join(&self.path)
        };

        // Build validation config
        let config = ValidationConfig::new(&workspace_root);

        // Create validator
        let mut validator = ArchitectureValidator::with_config(config);

        // Run validation
        let report = if let Some(ref validators) = self.validators {
            // Run specific validators
            let validator_names: Vec<&str> = validators.iter().map(String::as_str).collect();
            validator.validate_named(&validator_names)?
        } else {
            // Run all validators
            validator.validate_all()?
        };

        // Format output
        match self.format.as_str() {
            "json" => {
                self.print_json(&report)?;
            }
            _ => {
                self.print_text(&report);
            }
        }

        // Return counts for exit code
        Ok(ValidationResult {
            errors: report.summary.errors,
            warnings: report.summary.warnings,
            infos: report.summary.infos,
            strict_mode: self.strict,
        })
    }

    /// Print report as JSON
    fn print_json(
        &self,
        report: &mcb_validate::GenericReport,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(report)?;
        println!("{json}");
        Ok(())
    }

    /// Print report as text
    fn print_text(&self, report: &mcb_validate::GenericReport) {
        // Filter by severity threshold
        let severity_threshold = match self.severity.as_str() {
            "error" => 0,   // Only errors
            "warning" => 1, // Errors + warnings
            _ => 2,         // All (including info)
        };

        // Print violations (unless quick mode)
        if !self.quick {
            let mut has_violations = false;

            for violations in report.violations_by_category.values() {
                for violation in violations {
                    let sev_level = match violation.severity.as_str() {
                        "ERROR" => 0,
                        "WARNING" => 1,
                        _ => 2,
                    };

                    if sev_level <= severity_threshold {
                        has_violations = true;
                        let file_display = violation
                            .file
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| "-".to_string());
                        let line = violation.line.unwrap_or(0);

                        println!(
                            "[{}] {}: {} ({}:{})",
                            violation.severity, violation.id, violation.message, file_display, line
                        );
                        if let Some(ref suggestion) = violation.suggestion {
                            println!("  → {suggestion}");
                        }
                    }
                }
            }

            if has_violations {
                println!();
            }
        }

        // Print summary
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
