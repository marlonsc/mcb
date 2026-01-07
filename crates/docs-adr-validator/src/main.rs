//! ADR Compliance Validation Framework for Documentation Excellence v0.0.4
//!
//! This tool validates that architectural decisions documented in ADRs
//! are properly implemented in the codebase and maintained over time.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn, error};
use walkdir::WalkDir;

/// ADR Compliance Validator - v0.0.4 Documentation Excellence
#[derive(Parser)]
#[command(name = "adr-validator")]
#[command(about = "ADR Compliance Validation Framework for Documentation Excellence v0.0.4")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate all ADRs against current codebase
    Validate {
        /// Path to ADR directory (default: docs/adr)
        #[arg(short, long, default_value = "docs/adr")]
        adr_path: PathBuf,

        /// Path to source code directory (default: crates/mcp-context-browser/src)
        #[arg(short, long, default_value = "crates/mcp-context-browser/src")]
        source_path: PathBuf,

        /// Output format (json, text, summary)
        #[arg(short, long, default_value = "summary")]
        format: String,
    },

    /// Generate compliance report for specific ADR
    Check {
        /// ADR number to check
        adr_number: u32,

        /// Path to ADR directory
        #[arg(short, long, default_value = "docs/adr")]
        adr_path: PathBuf,
    },

    /// List all ADRs with compliance status
    List {
        /// Path to ADR directory
        #[arg(short, long, default_value = "docs/adr")]
        adr_path: PathBuf,

        /// Show detailed status
        #[arg(short, long)]
        detailed: bool,
    },
}

/// Architecture Decision Record structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArchitectureDecisionRecord {
    pub number: u32,
    pub title: String,
    pub status: AdrStatus,
    pub context: String,
    pub decision: String,
    pub consequences: Vec<String>,
    pub compliance_rules: Vec<ComplianceRule>,
}

/// ADR Status
#[derive(Debug, Clone, Serialize, Deserialize)]
enum AdrStatus {
    Proposed,
    Accepted,
    Rejected,
    Deprecated,
    Superseded,
}

/// Compliance Rule for ADR validation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComplianceRule {
    pub rule_type: ComplianceRuleType,
    pub description: String,
    pub patterns: Vec<String>,
    pub severity: ComplianceSeverity,
}

/// Types of compliance rules
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ComplianceRuleType {
    /// Code must contain specific patterns
    MustContain,
    /// Code must NOT contain specific patterns
    MustNotContain,
    /// File structure requirements
    FileStructure,
    /// Dependency requirements
    Dependencies,
    /// API contract requirements
    ApiContract,
}

/// Compliance severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ComplianceSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Compliance validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComplianceResult {
    pub adr_number: u32,
    pub adr_title: String,
    pub status: ComplianceStatus,
    pub violations: Vec<ComplianceViolation>,
    pub score: f64,
}

/// Overall compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ComplianceStatus {
    Compliant,
    Warning,
    Violation,
}

/// Specific compliance violation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComplianceViolation {
    pub rule: String,
    pub severity: ComplianceSeverity,
    pub description: String,
    pub locations: Vec<String>,
}

/// Overall compliance report
#[derive(Debug, Serialize, Deserialize)]
struct ComplianceReport {
    pub timestamp: String,
    pub total_adrs: usize,
    pub compliant_adrs: usize,
    pub warning_adrs: usize,
    pub violation_adrs: usize,
    pub overall_score: f64,
    pub results: Vec<ComplianceResult>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { adr_path, source_path, format } => {
            validate_adrs(adr_path, source_path, format).await?;
        }
        Commands::Check { adr_number, adr_path } => {
            check_adr(adr_number, adr_path).await?;
        }
        Commands::List { adr_path, detailed } => {
            list_adrs(adr_path, detailed).await?;
        }
    }

    Ok(())
}

/// Validate all ADRs against the current codebase
async fn validate_adrs(adr_path: PathBuf, source_path: PathBuf, format: String) -> Result<()> {
    info!("🔍 Starting ADR compliance validation...");
    info!("📁 ADR path: {}", adr_path.display());
    info!("📁 Source path: {}", source_path.display());

    // Load all ADRs
    let adrs = load_adrs(&adr_path).await?;
    info!("📋 Found {} ADRs to validate", adrs.len());

    // Validate each ADR
    let mut results = Vec::new();
    for adr in &adrs {
        let result = validate_single_adr(adr, &source_path).await?;
        results.push(result);
    }

    // Generate report
    let report = ComplianceReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        total_adrs: adrs.len(),
        compliant_adrs: results.iter().filter(|r| matches!(r.status, ComplianceStatus::Compliant)).count(),
        warning_adrs: results.iter().filter(|r| matches!(r.status, ComplianceStatus::Warning)).count(),
        violation_adrs: results.iter().filter(|r| matches!(r.status, ComplianceStatus::Violation)).count(),
        overall_score: calculate_overall_score(&results),
        results,
    };

    // Output report
    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        "text" => {
            print_text_report(&report);
        }
        "summary" => {
            print_summary_report(&report);
        }
        _ => {
            eprintln!("❌ Invalid format: {}. Use 'json', 'text', or 'summary'", format);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Load all ADRs from the ADR directory
async fn load_adrs(adr_path: &Path) -> Result<Vec<ArchitectureDecisionRecord>> {
    let mut adrs = Vec::new();

    for entry in WalkDir::new(adr_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                // Skip README.md and template files
                if file_name == "README.md" || file_name.contains("template") {
                    continue;
                }

                // Try to parse ADR number from filename
                if let Some(adr_number) = parse_adr_number(file_name) {
                    match parse_adr_file(path).await {
                        Ok(adr) => {
                            adrs.push(adr);
                        }
                        Err(e) => {
                            warn!("Failed to parse ADR {}: {}", file_name, e);
                        }
                    }
                }
            }
        }
    }

    // Sort by number
    adrs.sort_by_key(|adr| adr.number);

    Ok(adrs)
}

/// Parse ADR number from filename (e.g., "005-adr-title.md" -> 5)
fn parse_adr_number(filename: &str) -> Option<u32> {
    // Match patterns like "005-", "5-", etc.
    let re = regex::Regex::new(r"^(\d+)-").ok()?;
    re.captures(filename)?
        .get(1)?
        .as_str()
        .parse::<u32>()
        .ok()
}

/// Parse ADR file content
async fn parse_adr_file(path: &Path) -> Result<ArchitectureDecisionRecord> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read ADR file: {}", path.display()))?;

    // Extract ADR number from filename
    let file_name = path.file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

    let number = parse_adr_number(file_name)
        .ok_or_else(|| anyhow::anyhow!("Could not parse ADR number from filename: {}", file_name))?;

    // Extract title from first heading
    let title = extract_title(&content)?;

    // Extract status
    let status = extract_status(&content)?;

    // Extract sections
    let context = extract_section(&content, "Context")?;
    let decision = extract_section(&content, "Decision")?;
    let consequences = extract_consequences(&content)?;

    // Generate compliance rules based on ADR content
    let compliance_rules = generate_compliance_rules(&decision, &consequences);

    Ok(ArchitectureDecisionRecord {
        number,
        title,
        status,
        context,
        decision,
        consequences,
        compliance_rules,
    })
}

/// Extract title from ADR content
fn extract_title(content: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();

    for line in lines {
        if line.starts_with("# ") {
            return Ok(line.trim_start_matches("# ").to_string());
        }
    }

    Err(anyhow::anyhow!("No title found in ADR"))
}

/// Extract status from ADR content
fn extract_status(content: &str) -> Result<AdrStatus> {
    if content.contains("## Status\n\nAccepted") {
        Ok(AdrStatus::Accepted)
    } else if content.contains("## Status\n\nProposed") {
        Ok(AdrStatus::Proposed)
    } else if content.contains("## Status\n\nRejected") {
        Ok(AdrStatus::Rejected)
    } else if content.contains("## Status\n\nDeprecated") {
        Ok(AdrStatus::Deprecated)
    } else if content.contains("## Status\n\nSuperseded") {
        Ok(AdrStatus::Superseded)
    } else {
        Ok(AdrStatus::Proposed) // Default
    }
}

/// Extract section content from ADR
fn extract_section(content: &str, section_name: &str) -> Result<String> {
    let pattern = format!("## {}", section_name);
    let lines: Vec<&str> = content.lines().collect();

    let mut in_section = false;
    let mut section_content = String::new();

    for line in lines {
        if line == &pattern {
            in_section = true;
            continue;
        }

        if in_section {
            if line.starts_with("## ") && line != &pattern {
                break; // Next section
            }
            section_content.push_str(line);
            section_content.push('\n');
        }
    }

    Ok(section_content.trim().to_string())
}

/// Extract consequences from ADR
fn extract_consequences(content: &str) -> Result<Vec<String>> {
    let consequences_section = extract_section(content, "Consequences")?;
    let lines: Vec<&str> = consequences_section.lines().collect();

    let mut consequences = Vec::new();
    for line in lines {
        let line = line.trim();
        if line.starts_with("### ") {
            consequences.push(line.trim_start_matches("### ").to_string());
        } else if line.starts_with("- ") {
            consequences.push(line.trim_start_matches("- ").to_string());
        }
    }

    Ok(consequences)
}

/// Generate compliance rules based on ADR content
fn generate_compliance_rules(decision: &str, consequences: &[String]) -> Vec<ComplianceRule> {
    let mut rules = Vec::new();

    // Analyze decision text for patterns
    if decision.to_lowercase().contains("must use") {
        rules.push(ComplianceRule {
            rule_type: ComplianceRuleType::MustContain,
            description: "Required implementation pattern from ADR decision".to_string(),
            patterns: extract_patterns_from_text(decision),
            severity: ComplianceSeverity::High,
        });
    }

    if decision.to_lowercase().contains("must not") {
        rules.push(ComplianceRule {
            rule_type: ComplianceRuleType::MustNotContain,
            description: "Prohibited pattern from ADR decision".to_string(),
            patterns: extract_negative_patterns_from_text(decision),
            severity: ComplianceSeverity::Critical,
        });
    }

    // Analyze consequences for compliance requirements
    for consequence in consequences {
        if consequence.to_lowercase().contains("breaking change") {
            rules.push(ComplianceRule {
                rule_type: ComplianceRuleType::ApiContract,
                description: format!("Breaking change consequence: {}", consequence),
                patterns: vec![],
                severity: ComplianceSeverity::Critical,
            });
        }
    }

    rules
}

/// Extract patterns from decision text
fn extract_patterns_from_text(text: &str) -> Vec<String> {
    // Simple pattern extraction - can be enhanced
    vec![text.to_string()]
}

/// Extract negative patterns from decision text
fn extract_negative_patterns_from_text(text: &str) -> Vec<String> {
    // Simple pattern extraction - can be enhanced
    vec![text.to_string()]
}

/// Validate a single ADR against the codebase
async fn validate_single_adr(adr: &ArchitectureDecisionRecord, source_path: &Path) -> Result<ComplianceResult> {
    let mut violations = Vec::new();
    let mut score = 1.0;

    for rule in &adr.compliance_rules {
        let rule_violations = check_compliance_rule(rule, source_path).await?;
        violations.extend(rule_violations);

        // Adjust score based on violations
        for violation in &violations {
            match violation.severity {
                ComplianceSeverity::Critical => score *= 0.0,
                ComplianceSeverity::High => score *= 0.5,
                ComplianceSeverity::Medium => score *= 0.8,
                ComplianceSeverity::Low => score *= 0.9,
            }
        }
    }

    let status = if violations.iter().any(|v| matches!(v.severity, ComplianceSeverity::Critical)) {
        ComplianceStatus::Violation
    } else if violations.iter().any(|v| matches!(v.severity, ComplianceSeverity::High)) {
        ComplianceStatus::Warning
    } else {
        ComplianceStatus::Compliant
    };

    Ok(ComplianceResult {
        adr_number: adr.number,
        adr_title: adr.title.clone(),
        status,
        violations,
        score,
    })
}

/// Check a specific compliance rule against the codebase
async fn check_compliance_rule(rule: &ComplianceRule, source_path: &Path) -> Result<Vec<ComplianceViolation>> {
    let mut violations = Vec::new();

    match rule.rule_type {
        ComplianceRuleType::MustContain => {
            for pattern in &rule.patterns {
                if !check_pattern_exists(pattern, source_path) {
                    violations.push(ComplianceViolation {
                        rule: rule.description.clone(),
                        severity: rule.severity.clone(),
                        description: format!("Required pattern not found: {}", pattern),
                        locations: vec![],
                    });
                }
            }
        }
        ComplianceRuleType::MustNotContain => {
            for pattern in &rule.patterns {
                let locations = find_pattern_occurrences(pattern, source_path);
                if !locations.is_empty() {
                    violations.push(ComplianceViolation {
                        rule: rule.description.clone(),
                        severity: rule.severity.clone(),
                        description: format!("Prohibited pattern found: {}", pattern),
                        locations,
                    });
                }
            }
        }
        _ => {
            // Other rule types not implemented yet
            warn!("Rule type {:?} not implemented yet", rule.rule_type);
        }
    }

    Ok(violations)
}

/// Check if a pattern exists in the codebase
fn check_pattern_exists(pattern: &str, source_path: &Path) -> bool {
    for entry in WalkDir::new(source_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Ok(content) = fs::read_to_string(path) {
                if content.contains(pattern) {
                    return true;
                }
            }
        }
    }

    false
}

/// Find all occurrences of a pattern in the codebase
fn find_pattern_occurrences(pattern: &str, source_path: &Path) -> Vec<String> {
    let mut locations = Vec::new();

    for entry in WalkDir::new(source_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Ok(content) = fs::read_to_string(path) {
                if content.contains(pattern) {
                    locations.push(path.display().to_string());
                }
            }
        }
    }

    locations
}

/// Calculate overall compliance score
fn calculate_overall_score(results: &[ComplianceResult]) -> f64 {
    if results.is_empty() {
        return 1.0;
    }

    let total_score: f64 = results.iter().map(|r| r.score).sum();
    total_score / results.len() as f64
}

/// Print text report
fn print_text_report(report: &ComplianceReport) {
    println!("🔍 ADR Compliance Report");
    println!("📅 Generated: {}", report.timestamp);
    println!("📊 Summary:");
    println!("   Total ADRs: {}", report.total_adrs);
    println!("   Compliant: {} ({:.1}%)", report.compliant_adrs,
             report.compliant_adrs as f64 / report.total_adrs as f64 * 100.0);
    println!("   Warnings: {} ({:.1}%)", report.warning_adrs,
             report.warning_adrs as f64 / report.total_adrs as f64 * 100.0);
    println!("   Violations: {} ({:.1}%)", report.violation_adrs,
             report.violation_adrs as f64 / report.total_adrs as f64 * 100.0);
    println!("   Overall Score: {:.1}%", report.overall_score * 100.0);
    println!();

    for result in &report.results {
        let status_icon = match result.status {
            ComplianceStatus::Compliant => "✅",
            ComplianceStatus::Warning => "⚠️",
            ComplianceStatus::Violation => "❌",
        };

        println!("{} ADR {:03}: {} (Score: {:.1}%)",
                status_icon, result.adr_number, result.adr_title, result.score * 100.0);

        for violation in &result.violations {
            println!("   🔴 {}", violation.description);
            for location in &violation.locations {
                println!("      📁 {}", location);
            }
        }
    }
}

/// Print summary report
fn print_summary_report(report: &ComplianceReport) {
    println!("📊 ADR Compliance Summary");
    println!("Overall Score: {:.1}%", report.overall_score * 100.0);
    println!("Compliant: {}/{}", report.compliant_adrs, report.total_adrs);
    println!("Warnings: {}/{}", report.warning_adrs, report.total_adrs);
    println!("Violations: {}/{}", report.violation_adrs, report.total_adrs);

    if report.overall_score < 0.8 {
        println!("❌ Compliance issues detected!");
        std::process::exit(1);
    } else {
        println!("✅ All ADRs compliant!");
    }
}

/// Check specific ADR
async fn check_adr(adr_number: u32, adr_path: PathBuf) -> Result<()> {
    let adrs = load_adrs(&adr_path).await?;
    let adr = adrs.into_iter().find(|a| a.number == adr_number)
        .ok_or_else(|| anyhow::anyhow!("ADR {} not found", adr_number))?;

    println!("📋 ADR {:03}: {}", adr.number, adr.title);
    println!("📊 Status: {:?}", adr.status);
    println!("📝 Context: {}", adr.context.lines().next().unwrap_or("N/A"));
    println!("🎯 Decision: {}", adr.decision.lines().next().unwrap_or("N/A"));

    println!("🔍 Compliance Rules:");
    for rule in &adr.compliance_rules {
        println!("   - {} ({:?})", rule.description, rule.severity);
    }

    Ok(())
}

/// List all ADRs
async fn list_adrs(adr_path: PathBuf, detailed: bool) -> Result<()> {
    let adrs = load_adrs(&adr_path).await?;

    println!("📋 ADR Registry ({})", adrs.len());

    for adr in adrs {
        if detailed {
            println!("ADR {:03}: {} - Status: {:?}", adr.number, adr.title, adr.status);
            println!("   Rules: {}", adr.compliance_rules.len());
            println!();
        } else {
            println!("{:03}: {} ({:?})", adr.number, adr.title, adr.status);
        }
    }

    Ok(())
}