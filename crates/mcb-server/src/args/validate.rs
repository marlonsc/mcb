//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

tool_enum! {
/// Actions available for the validate tool.
pub enum ValidateAction {
    /// Run architectural validation rules.
    Run,
    /// List available validation rules.
    ListRules,
    /// Analyze code complexity (cyclomatic, cognitive).
    Analyze,
}
}

tool_enum! {
/// Scope for the validate action.
pub enum ValidateScope {
    /// Validate a single file.
    File,
    /// Validate an entire project.
    Project,
}
}

tool_schema! {
/// Arguments for the validate tool.
pub struct ValidateArgs {
    /// Action: run (validate), `list_rules`, analyze (complexity).
    #[schemars(description = "Action: run (validate), list_rules, analyze (complexity)")]
    pub action: ValidateAction,

    /// Scope: file or project.
    #[schemars(description = "Scope: file or project", with = "ValidateScope")]
    pub scope: Option<ValidateScope>,

    /// Path to file or project directory.
    #[schemars(description = "Path to file or project directory", with = "String")]
    pub path: Option<String>,

    /// Specific rules to run (empty = all).
    #[schemars(
        description = "Specific rules to run (empty = all)",
        with = "Vec<String>"
    )]
    pub rules: Option<Vec<String>>,

    /// Rule category filter.
    #[schemars(description = "Rule category filter", with = "String")]
    pub category: Option<String>,
}
}

// ---------------------------------------------------------------------------
// MCP-facing single-purpose tools
// ---------------------------------------------------------------------------

tool_action! {
    /// Arguments for the `validate_code` tool.
    pub struct ValidateCodeArgs => ValidateArgs {
        #[schemars(description = "Scope: file or project (default: project)", with = "ValidateScope")]
        scope: Option<ValidateScope>,
        #[schemars(description = "Specific rules to run (empty = all)", with = "Vec<String>")]
        rules: Option<Vec<String>>,
        #[schemars(description = "Rule category filter", with = "String")]
        category: Option<String>
        ;
        hidden { path: Option<String> }
        ;
        convert |a| { action: ValidateAction::Run, scope: a.scope, rules: a.rules, category: a.category }
    }
}

tool_action! {
    /// Arguments for the `analyze_code` tool.
    pub struct AnalyzeCodeArgs => ValidateArgs {
        #[schemars(description = "Path to file or directory", with = "String")]
        path: Option<String>
        ;
        hidden { }
        ;
        convert |a| { action: ValidateAction::Analyze, scope: None, path: a.path, rules: None, category: None }
    }
}

tool_action! {
    /// Arguments for the `list_rules` tool.
    pub struct ListRulesArgs => ValidateArgs {
        #[schemars(description = "Filter by category", with = "String")]
        category: Option<String>
        ;
        hidden { }
        ;
        convert |a| { action: ValidateAction::ListRules, scope: None, path: None, rules: None, category: a.category }
    }
}
