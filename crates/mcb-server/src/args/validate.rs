use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::macros::{tool_enum, tool_schema};

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
