use std::collections::{HashMap, HashSet};

use super::DependencyValidator;
use super::violation::{DependencyCycle, DependencyViolation};
use crate::constants::common::MCB_DEPENDENCY_PREFIX;
use crate::linters::constants::CARGO_TOML_FILENAME;
use crate::{Result, Severity};

/// Detect circular dependencies using topological sort
pub fn detect_circular_dependencies(
    validator: &DependencyValidator,
) -> Result<Vec<DependencyViolation>> {
    let mut violations = Vec::new();
    let mut graph: HashMap<String, HashSet<String>> = HashMap::new();

    // Build dependency graph from Cargo.toml files
    for crate_name in validator.allowed_deps.keys() {
        let cargo_toml = validator
            .config
            .workspace_root
            .join("crates")
            .join(crate_name)
            .join(CARGO_TOML_FILENAME);

        if !cargo_toml.exists() {
            continue;
        }

        let content = std::fs::read_to_string(&cargo_toml)?;
        let parsed: toml::Value = toml::from_str(&content)?;

        let mut deps = HashSet::new();
        if let Some(dependencies) = parsed.get("dependencies").and_then(|d| d.as_table()) {
            for dep_name in dependencies.keys() {
                if dep_name.starts_with(MCB_DEPENDENCY_PREFIX) {
                    deps.insert(dep_name.replace('_', "-"));
                }
            }
        }
        graph.insert(crate_name.clone(), deps);
    }

    // Detect cycles using DFS
    for start in graph.keys() {
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        if let Some(cycle) = find_cycle_impl(&graph, start, &mut visited, &mut path) {
            violations.push(DependencyViolation::CircularDependency {
                cycle: DependencyCycle(cycle),
                severity: Severity::Error,
            });
        }
    }

    Ok(violations)
}

/// Detects cycles in the dependency graph using depth-first search.
///
/// Recursively traverses the dependency graph to find circular dependencies.
/// Returns the cycle path if found, or None if no cycle exists from this node.
///
/// # Arguments
/// * `graph` - The dependency graph mapping crate names to their dependencies
/// * `node` - The current node being visited
/// * `visited` - Set of nodes already fully explored
/// * `path` - Current path being explored (used to detect cycles)
fn find_cycle_impl(
    graph: &HashMap<String, HashSet<String>>,
    node: &str,
    visited: &mut HashSet<String>,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    if path.contains(&node.to_owned()) {
        let cycle_start = path.iter().position(|n| n == node)?;
        let mut cycle: Vec<String> = path[cycle_start..].to_vec();
        cycle.push(node.to_owned());
        return Some(cycle);
    }

    if visited.contains(node) {
        return None;
    }

    visited.insert(node.to_owned());
    path.push(node.to_owned());

    if let Some(deps) = graph.get(node) {
        for dep in deps {
            if let Some(cycle) = find_cycle_impl(graph, dep, visited, path) {
                return Some(cycle);
            }
        }
    }

    path.pop();
    None
}
