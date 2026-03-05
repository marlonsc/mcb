use crate::EmbeddedRules;
use mcb_domain::ports::RuleInfo;
use mcb_utils::constants::validate::{
    CATEGORY_QUALITY, DEFAULT_RULE_DESCRIPTION, RUSTY_RULES, SEVERITY_WARNING, YAML_FIELD_BASE,
    YAML_FIELD_CATEGORY, YAML_FIELD_DESCRIPTION, YAML_FIELD_ENABLED, YAML_FIELD_ENGINE,
    YAML_FIELD_ID, YAML_FIELD_SEVERITY,
};

/// Get validation rules from embedded YAML files, optionally filtering by category.
#[must_use]
pub fn get_validation_rules(category: Option<&str>) -> Vec<RuleInfo> {
    let all_rules: Vec<RuleInfo> = EmbeddedRules::all_yaml()
        .into_iter()
        .filter(|(path, _)| {
            path.ends_with(".yml")
                && !path.contains("/templates/")
                && !path.starts_with("templates/")
        })
        .filter_map(|(_, content)| {
            if extract_yaml_scalar(content, YAML_FIELD_BASE).as_deref() == Some("true") {
                return None;
            }

            let enabled = extract_yaml_scalar(content, YAML_FIELD_ENABLED)
                .is_none_or(|value| value != "false");
            if !enabled {
                return None;
            }

            let id = extract_yaml_scalar(content, YAML_FIELD_ID)?;
            Some(RuleInfo {
                id,
                category: extract_yaml_scalar(content, YAML_FIELD_CATEGORY)
                    .unwrap_or_else(|| CATEGORY_QUALITY.to_owned()),
                severity: extract_yaml_scalar(content, YAML_FIELD_SEVERITY)
                    .unwrap_or_else(|| SEVERITY_WARNING.to_owned()),
                description: extract_yaml_scalar(content, YAML_FIELD_DESCRIPTION)
                    .unwrap_or_else(|| DEFAULT_RULE_DESCRIPTION.to_owned()),
                engine: extract_yaml_scalar(content, YAML_FIELD_ENGINE)
                    .unwrap_or_else(|| RUSTY_RULES.to_owned()),
            })
        })
        .collect();

    if let Some(cat) = category {
        all_rules
            .into_iter()
            .filter(|rule| rule.category == cat)
            .collect()
    } else {
        all_rules
    }
}

fn extract_yaml_scalar(content: &str, key: &str) -> Option<String> {
    let mapping: serde_yaml::Value = serde_yaml::from_str(content).ok()?;
    let value = mapping.get(key)?;

    match value {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        serde_yaml::Value::Null
        | serde_yaml::Value::Sequence(_)
        | serde_yaml::Value::Mapping(_)
        | serde_yaml::Value::Tagged(_) => None,
    }
}
