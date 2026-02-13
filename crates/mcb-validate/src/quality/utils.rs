pub fn has_ignore_hint(line: &str, violation_type: &str) -> bool {
    line.contains(&format!("mcb-validate-ignore: {violation_type}"))
}
