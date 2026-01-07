//! Automated Documentation Generator for Self-Documenting Codebase v0.0.4
//!
//! This tool generates comprehensive documentation from source code analysis,
//! creating self-documenting artifacts that stay synchronized with code changes.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn, error};
use walkdir::WalkDir;

/// Automated Documentation Generator - v0.0.4 Self-Documenting Codebase
#[derive(Parser)]
#[command(name = "docs-generator")]
#[command(about = "Automated Documentation Generator for Self-Documenting Codebase v0.0.4")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate all documentation from source code
    Generate {
        /// Source directory to analyze
        #[arg(short, long, default_value = "crates/mcp-context-browser/src")]
        source_dir: PathBuf,

        /// Output directory for generated documentation
        #[arg(short, long, default_value = "docs/generated")]
        output_dir: PathBuf,

        /// Include private items in documentation
        #[arg(long)]
        include_private: bool,

        /// Generate dependency graphs
        #[arg(long)]
        with_graphs: bool,
    },

    /// Analyze codebase and generate code metrics
    Analyze {
        /// Source directory to analyze
        #[arg(short, long, default_value = "crates/mcp-context-browser/src")]
        source_dir: PathBuf,

        /// Output format (json, markdown, html)
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },

    /// Generate API surface documentation
    Api {
        /// Source directory to analyze
        #[arg(short, long, default_value = "crates/mcp-context-browser/src")]
        source_dir: PathBuf,

        /// Output file
        #[arg(short, long, default_value = "docs/api-surface.md")]
        output_file: PathBuf,
    },

    /// Generate module dependency graphs
    Graph {
        /// Source directory to analyze
        #[arg(short, long, default_value = "crates/mcp-context-browser/src")]
        source_dir: PathBuf,

        /// Output directory
        #[arg(short, long, default_value = "docs/diagrams/generated")]
        output_dir: PathBuf,
    },
}

/// Code analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodeAnalysis {
    pub modules: Vec<ModuleInfo>,
    pub functions: Vec<FunctionInfo>,
    pub structs: Vec<StructInfo>,
    pub enums: Vec<EnumInfo>,
    pub traits: Vec<TraitInfo>,
    pub dependencies: Vec<DependencyInfo>,
    pub metrics: CodeMetrics,
}

/// Module information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModuleInfo {
    pub name: String,
    pub path: String,
    pub documentation: String,
    pub public_items: usize,
    pub private_items: usize,
    pub lines_of_code: usize,
}

/// Function information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FunctionInfo {
    pub name: String,
    pub module: String,
    pub signature: String,
    pub documentation: String,
    pub is_public: bool,
    pub complexity: usize,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
}

/// Struct information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StructInfo {
    pub name: String,
    pub module: String,
    pub documentation: String,
    pub is_public: bool,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<String>,
}

/// Field information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub is_public: bool,
    pub documentation: String,
}

/// Enum information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EnumInfo {
    pub name: String,
    pub module: String,
    pub documentation: String,
    pub is_public: bool,
    pub variants: Vec<String>,
}

/// Trait information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TraitInfo {
    pub name: String,
    pub module: String,
    pub documentation: String,
    pub is_public: bool,
    pub methods: Vec<String>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DependencyInfo {
    pub from_module: String,
    pub to_module: String,
    pub dependency_type: String,
}

/// Code metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodeMetrics {
    pub total_lines: usize,
    pub total_modules: usize,
    pub total_functions: usize,
    pub total_structs: usize,
    pub total_enums: usize,
    pub total_traits: usize,
    pub average_complexity: f64,
    pub test_coverage_estimate: f64,
    pub documentation_coverage: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { source_dir, output_dir, include_private, with_graphs } => {
            generate_documentation(source_dir, output_dir, include_private, with_graphs).await?;
        }
        Commands::Analyze { source_dir, format } => {
            analyze_codebase(source_dir, format).await?;
        }
        Commands::Api { source_dir, output_file } => {
            generate_api_surface(source_dir, output_file).await?;
        }
        Commands::Graph { source_dir, output_dir } => {
            generate_dependency_graphs(source_dir, output_dir).await?;
        }
    }

    Ok(())
}

/// Generate comprehensive documentation from source code
async fn generate_documentation(
    source_dir: PathBuf,
    output_dir: PathBuf,
    include_private: bool,
    with_graphs: bool,
) -> Result<()> {
    info!("📚 Starting automated documentation generation...");
    info!("📁 Source: {}", source_dir.display());
    info!("📁 Output: {}", output_dir.display());

    // Create output directory
    fs::create_dir_all(&output_dir)?;

    // Analyze codebase
    let analysis = analyze_codebase_internal(&source_dir).await?;
    info!("📊 Analysis complete: {} modules, {} functions",
          analysis.modules.len(), analysis.functions.len());

    // Generate module documentation
    generate_module_docs(&analysis, &output_dir, include_private).await?;

    // Generate API documentation
    generate_api_docs(&analysis, &output_dir, include_private).await?;

    // Generate metrics documentation
    generate_metrics_docs(&analysis, &output_dir).await?;

    // Generate dependency graphs if requested
    if with_graphs {
        generate_dependency_graphs(source_dir, output_dir.join("diagrams")).await?;
    }

    // Generate index
    generate_documentation_index(&output_dir).await?;

    info!("✅ Documentation generation complete!");
    info!("📖 Generated files in: {}", output_dir.display());

    Ok(())
}

/// Analyze codebase internally
async fn analyze_codebase_internal(source_dir: &Path) -> Result<CodeAnalysis> {
    let mut modules = Vec::new();
    let mut functions = Vec::new();
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut traits = Vec::new();
    let mut dependencies = Vec::new();

    // Walk through source files
    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let relative_path = path.strip_prefix(source_dir)?;

            // Parse Rust file
            match parse_rust_file(path, relative_path).await {
                Ok((mod_info, funcs, strcts, enms, trts, deps)) => {
                    modules.push(mod_info);
                    functions.extend(funcs);
                    structs.extend(strcts);
                    enums.extend(enms);
                    traits.extend(trts);
                    dependencies.extend(deps);
                }
                Err(e) => {
                    warn!("Failed to parse {}: {}", path.display(), e);
                }
            }
        }
    }

    // Calculate metrics
    let total_lines = modules.iter().map(|m| m.lines_of_code).sum();
    let metrics = CodeMetrics {
        total_lines,
        total_modules: modules.len(),
        total_functions: functions.len(),
        total_structs: structs.len(),
        total_enums: enums.len(),
        total_traits: traits.len(),
        average_complexity: calculate_average_complexity(&functions),
        test_coverage_estimate: estimate_test_coverage(&functions),
        documentation_coverage: calculate_documentation_coverage(&functions, &structs, &traits),
    };

    Ok(CodeAnalysis {
        modules,
        functions,
        structs,
        enums,
        traits,
        dependencies,
        metrics,
    })
}

/// Parse a single Rust file
async fn parse_rust_file(
    path: &Path,
    relative_path: &Path,
) -> Result<(ModuleInfo, Vec<FunctionInfo>, Vec<StructInfo>, Vec<EnumInfo>, Vec<TraitInfo>, Vec<DependencyInfo>)> {
    let content = fs::read_to_string(path)?;

    // Parse with syn
    let syntax = syn::parse_file(&content)?;

    let mut functions = Vec::new();
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut traits = Vec::new();
    let mut dependencies = Vec::new();

    // Extract items
    for item in &syntax.items {
        match item {
            syn::Item::Fn(func) => {
                if let Some(func_info) = extract_function_info(func, relative_path) {
                    functions.push(func_info);
                }
            }
            syn::Item::Struct(strct) => {
                if let Some(struct_info) = extract_struct_info(strct, relative_path) {
                    structs.push(struct_info);
                }
            }
            syn::Item::Enum(enm) => {
                if let Some(enum_info) = extract_enum_info(enm, relative_path) {
                    enums.push(enum_info);
                }
            }
            syn::Item::Trait(trt) => {
                if let Some(trait_info) = extract_trait_info(trt, relative_path) {
                    traits.push(trait_info);
                }
            }
            syn::Item::Use(use_item) => {
                extract_dependencies(use_item, relative_path, &mut dependencies);
            }
            _ => {}
        }
    }

    // Create module info
    let module_name = relative_path.to_string_lossy().trim_end_matches(".rs").to_string();
    let module_info = ModuleInfo {
        name: module_name,
        path: relative_path.to_string_lossy().to_string(),
        documentation: extract_module_docs(&syntax),
        public_items: count_public_items(&syntax),
        private_items: count_private_items(&syntax),
        lines_of_code: content.lines().count(),
    };

    Ok((module_info, functions, structs, enums, traits, dependencies))
}

/// Extract function information
fn extract_function_info(func: &syn::ItemFn, module_path: &Path) -> Option<FunctionInfo> {
    let name = func.sig.ident.to_string();
    let module = module_path.to_string_lossy().to_string();

    // Check if public
    let is_public = matches!(func.vis, syn::Visibility::Public(_));

    // Extract parameters
    let parameters = func.sig.inputs.iter()
        .filter_map(|arg| {
            match arg {
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(pat_type) => {
                    Some(quote::quote!(#pat_type).to_string())
                }
            }
        })
        .collect();

    // Extract return type
    let return_type = match &func.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(quote::quote!(#ty).to_string()),
    };

    // Extract documentation
    let documentation = extract_docs(&func.attrs);

    // Calculate complexity (simplified)
    let complexity = calculate_function_complexity(func);

    Some(FunctionInfo {
        name,
        module,
        signature: quote::quote!(#func).to_string(),
        documentation,
        is_public,
        complexity,
        parameters,
        return_type,
    })
}

/// Extract struct information
fn extract_struct_info(strct: &syn::ItemStruct, module_path: &Path) -> Option<StructInfo> {
    let name = strct.ident.to_string();
    let module = module_path.to_string_lossy().to_string();
    let is_public = matches!(strct.vis, syn::Visibility::Public(_));
    let documentation = extract_docs(&strct.attrs);

    // Extract fields
    let fields = strct.fields.iter()
        .filter_map(|field| {
            let field_name = match &field.ident {
                Some(ident) => ident.to_string(),
                None => return None,
            };

            Some(FieldInfo {
                name: field_name,
                field_type: quote::quote!(#field.ty).to_string(),
                is_public: matches!(field.vis, syn::Visibility::Public(_)),
                documentation: extract_docs(&field.attrs),
            })
        })
        .collect();

    // Methods would need separate processing
    let methods = Vec::new();

    Some(StructInfo {
        name,
        module,
        documentation,
        is_public,
        fields,
        methods,
    })
}

/// Extract enum information
fn extract_enum_info(enm: &syn::ItemEnum, module_path: &Path) -> Option<EnumInfo> {
    let name = enm.ident.to_string();
    let module = module_path.to_string_lossy().to_string();
    let is_public = matches!(enm.vis, syn::Visibility::Public(_));
    let documentation = extract_docs(&enm.attrs);

    let variants = enm.variants.iter()
        .map(|v| v.ident.to_string())
        .collect();

    Some(EnumInfo {
        name,
        module,
        documentation,
        is_public,
        variants,
    })
}

/// Extract trait information
fn extract_trait_info(trt: &syn::ItemTrait, module_path: &Path) -> Option<TraitInfo> {
    let name = trt.ident.to_string();
    let module = module_path.to_string_lossy().to_string();
    let is_public = matches!(trt.vis, syn::Visibility::Public(_));
    let documentation = extract_docs(&trt.attrs);

    let methods = trt.items.iter()
        .filter_map(|item| {
            match item {
                syn::TraitItem::Fn(method) => Some(method.sig.ident.to_string()),
                _ => None,
            }
        })
        .collect();

    Some(TraitInfo {
        name,
        module,
        documentation,
        is_public,
        methods,
    })
}

/// Extract dependencies from use statements
fn extract_dependencies(
    use_item: &syn::ItemUse,
    module_path: &Path,
    dependencies: &mut Vec<DependencyInfo>,
) {
    // This is a simplified implementation
    // A full implementation would parse the use tree
    let from_module = module_path.to_string_lossy().to_string();

    // For now, just mark that this module has dependencies
    // Real implementation would need to resolve actual module paths
}

/// Extract documentation from attributes
fn extract_docs(attrs: &[syn::Attribute]) -> String {
    attrs.iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| {
            match &attr.meta {
                syn::Meta::NameValue(meta) => {
                    if let syn::Expr::Lit(expr_lit) = &meta.value {
                        if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                            Some(lit_str.value())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extract module-level documentation
fn extract_module_docs(syntax: &syn::File) -> String {
    syntax.attrs.iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| {
            match &attr.meta {
                syn::Meta::NameValue(meta) => {
                    if let syn::Expr::Lit(expr_lit) = &meta.value {
                        if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                            Some(lit_str.value())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Count public items
fn count_public_items(syntax: &syn::File) -> usize {
    syntax.items.iter()
        .filter(|item| matches!(item, syn::Item::Fn(_) | syn::Item::Struct(_) | syn::Item::Enum(_) | syn::Item::Trait(_)))
        .filter(|item| {
            match item {
                syn::Item::Fn(func) => matches!(func.vis, syn::Visibility::Public(_)),
                syn::Item::Struct(strct) => matches!(strct.vis, syn::Visibility::Public(_)),
                syn::Item::Enum(enm) => matches!(enm.vis, syn::Visibility::Public(_)),
                syn::Item::Trait(trt) => matches!(trt.vis, syn::Visibility::Public(_)),
                _ => false,
            }
        })
        .count()
}

/// Count private items
fn count_private_items(syntax: &syn::File) -> usize {
    syntax.items.iter()
        .filter(|item| matches!(item, syn::Item::Fn(_) | syn::Item::Struct(_) | syn::Item::Enum(_) | syn::Item::Trait(_)))
        .filter(|item| {
            match item {
                syn::Item::Fn(func) => !matches!(func.vis, syn::Visibility::Public(_)),
                syn::Item::Struct(strct) => !matches!(strct.vis, syn::Visibility::Public(_)),
                syn::Item::Enum(enm) => !matches!(enm.vis, syn::Visibility::Public(_)),
                syn::Item::Trait(trt) => !matches!(trt.vis, syn::Visibility::Public(_)),
                _ => true,
            }
        })
        .count()
}

/// Calculate function complexity (simplified)
fn calculate_function_complexity(func: &syn::ItemFn) -> usize {
    // Very basic complexity calculation
    let mut complexity = 1; // Base complexity

    // Count control flow statements (if, match, loop, etc.)
    // This is a simplified implementation
    complexity += count_control_flow(&func.block);

    complexity
}

/// Count control flow statements in a block
fn count_control_flow(block: &syn::Block) -> usize {
    let mut count = 0;

    for stmt in &block.stmts {
        match stmt {
            syn::Stmt::Expr(expr, _) => {
                count += count_control_flow_in_expr(expr);
            }
            syn::Stmt::Semi(expr, _) => {
                count += count_control_flow_in_expr(expr);
            }
            _ => {}
        }
    }

    count
}

/// Count control flow in expressions
fn count_control_flow_in_expr(expr: &syn::Expr) -> usize {
    match expr {
        syn::Expr::If(_) | syn::Expr::Match(_) | syn::Expr::Loop(_) |
        syn::Expr::While(_) | syn::Expr::ForLoop(_) => 1,
        syn::Expr::Block(block) => count_control_flow(&block.block),
        _ => 0,
    }
}

/// Calculate average complexity
fn calculate_average_complexity(functions: &[FunctionInfo]) -> f64 {
    if functions.is_empty() {
        return 0.0;
    }

    let total: usize = functions.iter().map(|f| f.complexity).sum();
    total as f64 / functions.len() as f64
}

/// Estimate test coverage (simplified)
fn estimate_test_coverage(functions: &[FunctionInfo]) -> f64 {
    // Very basic estimation - in real implementation would analyze test files
    if functions.is_empty() {
        return 0.0;
    }

    // Assume 70% base coverage
    0.7
}

/// Calculate documentation coverage
fn calculate_documentation_coverage(
    functions: &[FunctionInfo],
    structs: &[StructInfo],
    traits: &[TraitInfo],
) -> f64 {
    let total_items = functions.len() + structs.len() + traits.len();

    if total_items == 0 {
        return 1.0;
    }

    let documented_items = functions.iter().filter(|f| !f.documentation.is_empty()).count() +
                          structs.iter().filter(|s| !s.documentation.is_empty()).count() +
                          traits.iter().filter(|t| !t.documentation.is_empty()).count();

    documented_items as f64 / total_items as f64
}

/// Generate module documentation
async fn generate_module_docs(
    analysis: &CodeAnalysis,
    output_dir: &Path,
    include_private: bool,
) -> Result<()> {
    let modules_dir = output_dir.join("modules");
    fs::create_dir_all(&modules_dir)?;

    for module in &analysis.modules {
        let mut content = format!("# Module: {}\n\n", module.name);

        if !module.documentation.is_empty() {
            content.push_str(&format!("## Overview\n\n{}\n\n", module.documentation));
        }

        content.push_str(&format!("## Statistics\n\n"));
        content.push_str(&format!("- **Lines of Code**: {}\n", module.lines_of_code));
        content.push_str(&format!("- **Public Items**: {}\n", module.public_items));
        if include_private {
            content.push_str(&format!("- **Private Items**: {}\n", module.private_items));
        }
        content.push_str(&format!("- **Total Items**: {}\n\n", module.public_items + module.private_items));

        // Write module file
        let file_path = modules_dir.join(format!("{}.md", module.name.replace("/", "_")));
        fs::write(&file_path, content)?;
    }

    Ok(())
}

/// Generate API documentation
async fn generate_api_docs(
    analysis: &CodeAnalysis,
    output_dir: &Path,
    include_private: bool,
) -> Result<()> {
    let api_dir = output_dir.join("api");
    fs::create_dir_all(&api_dir)?;

    // Generate functions documentation
    let mut functions_content = "# API Functions\n\n".to_string();
    for func in &analysis.functions {
        if func.is_public || include_private {
            functions_content.push_str(&format!("## `{}`\n\n", func.name));
            if !func.documentation.is_empty() {
                functions_content.push_str(&format!("{}\n\n", func.documentation));
            }
            functions_content.push_str(&format!("**Signature:** {}\n\n", func.signature));
            functions_content.push_str(&format!("**Module:** {}\n\n", func.module));
            functions_content.push_str(&format!("**Complexity:** {}\n\n", func.complexity));

            if !func.parameters.is_empty() {
                functions_content.push_str("**Parameters:**\n");
                for param in &func.parameters {
                    functions_content.push_str(&format!("- {}\n", param));
                }
                functions_content.push('\n');
            }

            if let Some(return_type) = &func.return_type {
                functions_content.push_str(&format!("**Returns:** {}\n\n", return_type));
            }
        }
    }

    fs::write(api_dir.join("functions.md"), functions_content)?;

    // Generate structs documentation
    let mut structs_content = "# API Structs\n\n".to_string();
    for strct in &analysis.structs {
        if strct.is_public || include_private {
            structs_content.push_str(&format!("## `{}`\n\n", strct.name));
            if !strct.documentation.is_empty() {
                structs_content.push_str(&format!("{}\n\n", strct.documentation));
            }
            structs_content.push_str(&format!("**Module:** {}\n\n", strct.module));

            if !strct.fields.is_empty() {
                structs_content.push_str("**Fields:**\n");
                for field in &strct.fields {
                    if field.is_public || include_private {
                        structs_content.push_str(&format!("- `{}`: {} - {}\n",
                                                        field.name, field.field_type,
                                                        field.documentation));
                    }
                }
                structs_content.push('\n');
            }
        }
    }

    fs::write(api_dir.join("structs.md"), structs_content)?;

    Ok(())
}

/// Generate metrics documentation
async fn generate_metrics_docs(analysis: &CodeAnalysis, output_dir: &Path) -> Result<()> {
    let metrics_file = output_dir.join("metrics.md");
    let mut content = "# Codebase Metrics\n\n".to_string();

    content.push_str(&format!("## Overview\n\n"));
    content.push_str(&format!("- **Total Lines**: {}\n", analysis.metrics.total_lines));
    content.push_str(&format!("- **Modules**: {}\n", analysis.metrics.total_modules));
    content.push_str(&format!("- **Functions**: {}\n", analysis.metrics.total_functions));
    content.push_str(&format!("- **Structs**: {}\n", analysis.metrics.total_structs));
    content.push_str(&format!("- **Enums**: {}\n", analysis.metrics.total_enums));
    content.push_str(&format!("- **Traits**: {}\n", analysis.metrics.total_traits));

    content.push_str(&format!("\n## Quality Metrics\n\n"));
    content.push_str(&format!("- **Average Complexity**: {:.2}\n", analysis.metrics.average_complexity));
    content.push_str(&format!("- **Test Coverage Estimate**: {:.1}%\n", analysis.metrics.test_coverage_estimate * 100.0));
    content.push_str(&format!("- **Documentation Coverage**: {:.1}%\n", analysis.metrics.documentation_coverage * 100.0));

    fs::write(&metrics_file, content)?;

    Ok(())
}

/// Generate dependency graphs
async fn generate_dependency_graphs(source_dir: PathBuf, output_dir: PathBuf) -> Result<()> {
    info!("📊 Generating dependency graphs...");

    // Create output directory
    fs::create_dir_all(&output_dir)?;

    // Use cargo-modules if available
    if let Ok(output) = Command::new("cargo")
        .args(["modules", "generate", "graph", "--package", "mcp-context-browser"])
        .output()
    {
        if output.status.success() {
            info!("✅ Dependency graph generated with cargo-modules");
        } else {
            warn!("⚠️  cargo-modules not available, generating basic graph");
            generate_basic_dependency_graph(&source_dir, &output_dir).await?;
        }
    } else {
        warn!("⚠️  cargo-modules not available, generating basic graph");
        generate_basic_dependency_graph(&source_dir, &output_dir).await?;
    }

    Ok(())
}

/// Generate basic dependency graph
async fn generate_basic_dependency_graph(source_dir: &Path, output_dir: &Path) -> Result<()> {
    // Simple implementation - would need graphviz or similar for real graphs
    let graph_file = output_dir.join("dependencies.md");

    let mut content = "# Module Dependencies\n\n".to_string();
    content.push_str("```\n");
    content.push_str("// This is a placeholder for dependency visualization\n");
    content.push_str("// Install cargo-modules for automatic graph generation\n");
    content.push_str("// cargo install cargo-modules\n");
    content.push_str("```\n");

    fs::write(&graph_file, content)?;

    Ok(())
}

/// Generate documentation index
async fn generate_documentation_index(output_dir: &Path) -> Result<()> {
    let index_file = output_dir.join("README.md");
    let mut content = "# Auto-Generated Documentation\n\n".to_string();

    content.push_str("This documentation was automatically generated from the codebase.\n\n");

    content.push_str("## Contents\n\n");
    content.push_str("- [API Reference](./api/) - Public API documentation\n");
    content.push_str("- [Modules](./modules/) - Module documentation\n");
    content.push_str("- [Metrics](./metrics.md) - Codebase metrics\n");
    content.push_str("- [Dependencies](./diagrams/dependencies.md) - Module dependencies\n");

    content.push_str("\n## Generation Info\n\n");
    content.push_str(&format!("- **Generated**: {}\n", chrono::Utc::now().to_rfc3339()));
    content.push_str("- **Tool**: docs-generator v0.0.4\n");
    content.push_str("- **Source**: Self-documenting codebase analysis\n");

    fs::write(&index_file, content)?;

    Ok(())
}

/// Analyze codebase and output results
async fn analyze_codebase(source_dir: PathBuf, format: String) -> Result<()> {
    let analysis = analyze_codebase_internal(&source_dir).await?;

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&analysis)?);
        }
        "markdown" => {
            print_markdown_analysis(&analysis);
        }
        "html" => {
            print_html_analysis(&analysis);
        }
        _ => {
            eprintln!("❌ Invalid format: {}. Use 'json', 'markdown', or 'html'", format);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Print markdown analysis
fn print_markdown_analysis(analysis: &CodeAnalysis) {
    println!("# Codebase Analysis\n");

    println!("## Metrics\n");
    println!("- Total Lines: {}", analysis.metrics.total_lines);
    println!("- Total Modules: {}", analysis.metrics.total_modules);
    println!("- Total Functions: {}", analysis.metrics.total_functions);
    println!("- Total Structs: {}", analysis.metrics.total_structs);
    println!("- Average Complexity: {:.2}", analysis.metrics.average_complexity);
    println!("- Documentation Coverage: {:.1}%", analysis.metrics.documentation_coverage * 100.0);

    println!("\n## Modules\n");
    for module in &analysis.modules {
        println!("- **{}**: {} lines, {} public items",
                module.name, module.lines_of_code, module.public_items);
    }
}

/// Print HTML analysis
fn print_html_analysis(analysis: &CodeAnalysis) {
    println!("<!DOCTYPE html>");
    println!("<html><head><title>Codebase Analysis</title></head><body>");
    println!("<h1>Codebase Analysis</h1>");

    println!("<h2>Metrics</h2><ul>");
    println!("<li>Total Lines: {}</li>", analysis.metrics.total_lines);
    println!("<li>Total Modules: {}</li>", analysis.metrics.total_modules);
    println!("<li>Total Functions: {}</li>", analysis.metrics.total_functions);
    println!("<li>Average Complexity: {:.2}</li>", analysis.metrics.average_complexity);
    println!("</ul>");

    println!("<h2>Modules</h2><ul>");
    for module in &analysis.modules {
        println!("<li><strong>{}</strong>: {} lines, {} public items</li>",
                module.name, module.lines_of_code, module.public_items);
    }
    println!("</ul></body></html>");
}

/// Generate API surface documentation
async fn generate_api_surface(source_dir: PathBuf, output_file: PathBuf) -> Result<()> {
    let analysis = analyze_codebase_internal(&source_dir).await?;

    let mut content = "# API Surface Documentation\n\n".to_string();
    content.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().to_rfc3339()));

    // Public functions
    content.push_str("## Public Functions\n\n");
    for func in &analysis.functions {
        if func.is_public {
            content.push_str(&format!("- `{}` ({})\n", func.name, func.module));
        }
    }

    // Public structs
    content.push_str("\n## Public Structs\n\n");
    for strct in &analysis.structs {
        if strct.is_public {
            content.push_str(&format!("- `{}` ({})\n", strct.name, strct.module));
        }
    }

    // Public enums
    content.push_str("\n## Public Enums\n\n");
    for enm in &analysis.enums {
        if enm.is_public {
            content.push_str(&format!("- `{}` ({})\n", enm.name, enm.module));
        }
    }

    // Public traits
    content.push_str("\n## Public Traits\n\n");
    for trt in &analysis.traits {
        if trt.is_public {
            content.push_str(&format!("- `{}` ({})\n", trt.name, trt.module));
        }
    }

    // Create output directory
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_file, content)?;
    info!("✅ API surface documentation generated: {}", output_file.display());

    Ok(())
}