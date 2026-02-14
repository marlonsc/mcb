use regex::Regex;

use super::{QualityValidator, QualityViolation};
use crate::constants::{
    PENDING_LABEL_FIXME, PENDING_LABEL_HACK, PENDING_LABEL_TODO, PENDING_LABEL_XXX,
};
use crate::scan::for_each_scan_rs_path;
use crate::{Result, Severity};

/// Scans for pending task comments matching `PENDING_LABEL_*` constants.
pub fn validate(validator: &QualityValidator) -> Result<Vec<QualityViolation>> {
    let todo_pattern = Regex::new(&format!(
        r"(?i)({PENDING_LABEL_TODO}|{PENDING_LABEL_FIXME}|{PENDING_LABEL_XXX}|{PENDING_LABEL_HACK}):?\s*(.*)"
    ))?;

    let mut violations = Vec::new();
    for_each_scan_rs_path(&validator.config, false, |path, _src_dir| {
        if path.extension().is_none_or(|ext| ext != "rs") || !path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;

        for (line_num, line) in content.lines().enumerate() {
            if let Some(cap) = todo_pattern.captures(line) {
                let todo_type = cap.get(1).map_or("", |m| m.as_str());
                let message = cap.get(2).map_or("", |m| m.as_str()).trim();

                violations.push(QualityViolation::TodoComment {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    content: format!("{}: {}", todo_type.to_uppercase(), message),
                    severity: Severity::Info,
                });
            }
        }

        Ok(())
    })?;

    Ok(violations)
}
