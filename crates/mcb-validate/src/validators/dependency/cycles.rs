//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use std::collections::{HashMap, HashSet};

use super::DependencyValidator;
use super::violation::{DependencyCycle, DependencyViolation};
use crate::{Result, Severity};
use mcb_utils::constants::validate::CARGO_TOML_FILENAME;
use mcb_utils::constants::validate::MCB_DEPENDENCY_PREFIX;

/// Detect circular dependencies using topological sort
pub fn detect_circular_dependencies(
    validator: &DependencyValidator,
) -> Result<Vec<DependencyViolation>> {
    let mut violations = Vec::new();
    let graph = build_dependency_graph(validator)?;

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

/// Build the inter-crate dependency graph from each crate's `Cargo.toml`,
/// keeping only `mcb-` workspace dependencies.
///
/// # Errors
///
/// Returns an error if a `Cargo.toml` cannot be read or parsed.
fn build_dependency_graph(
    validator: &DependencyValidator,
) -> Result<HashMap<String, HashSet<String>>> {
    let mut graph: HashMap<String, HashSet<String>> = HashMap::new();

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

        graph.insert(crate_name.clone(), read_mcb_dependencies(&cargo_toml)?);
    }

    Ok(graph)
}

/// Parse a crate's `Cargo.toml` and return its internal (`mcb-*`) dependency names.
fn read_mcb_dependencies(cargo_toml: &std::path::Path) -> Result<HashSet<String>> {
    let content = std::fs::read_to_string(cargo_toml)?;
    let parsed: toml::Value = toml::from_str(&content)?;

    let Some(dependencies) = parsed.get("dependencies").and_then(|d| d.as_table()) else {
        return Ok(HashSet::new());
    };

    Ok(dependencies
        .keys()
        .filter(|name| name.starts_with(MCB_DEPENDENCY_PREFIX))
        .map(|name| name.replace('_', "-"))
        .collect())
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
