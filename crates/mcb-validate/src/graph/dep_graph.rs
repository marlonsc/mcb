//! Dependency graph construction and layer-boundary analysis.
//!
//! Builds a `petgraph` directed graph from extracted [`Fact`]s, enabling
//! cycle detection and inter-layer dependency checks.

use crate::extractor::{Fact, FactType};
use petgraph::Direction;
use petgraph::algo::tarjan_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// Type of edge in the dependency graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeType {
    /// The source fact depends on the target fact (e.g. via an import).
    DependsOn,
    /// The source fact contains the target fact (e.g. a module containing a struct).
    Contains,
}

/// A directed dependency graph built from code facts.
#[derive(Debug, Default)]
pub struct DependencyGraph {
    graph: DiGraph<Fact, EdgeType>,
    node_map: HashMap<String, NodeIndex>,
}

impl DependencyGraph {
    /// Create an empty dependency graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Populate the graph from extracted facts.
    ///
    /// Performs two passes:
    /// 1. Adds all facts as nodes.
    /// 2. Adds `Contains` edges (parent → child) and `DependsOn` edges
    ///    (import → target module) using heuristic name matching.
    pub fn build(&mut self, facts: &[Fact]) {
        // First pass: Add all nodes
        for fact in facts {
            if !self.node_map.contains_key(&fact.id) {
                let idx = self.graph.add_node(fact.clone());
                self.node_map.insert(fact.id.clone(), idx);
            }
        }

        // Second pass: Add edges
        for fact in facts {
            let Some(source_idx) = self.node_map.get(&fact.id).copied() else {
                continue;
            };

            // Add "Contains" edges (Parent -> Child)
            if let Some(parent_id) = &fact.parent_id
                && let Some(parent_idx) = self.node_map.get(parent_id)
            {
                self.graph
                    .add_edge(*parent_idx, source_idx, EdgeType::Contains);
            }

            // Add "DependsOn" edges (Import -> Target Module)
            if fact.fact_type == FactType::Import {
                let import_path = &fact.name;

                // Simple exact match check first
                if let Some(target_idx) = self.node_map.get(import_path) {
                    self.graph
                        .add_edge(source_idx, *target_idx, EdgeType::DependsOn);
                }

                // Check for module prefix (e.g. import "mcb_domain::..." depends on module "mcb_domain")
                for idx in self.node_map.values() {
                    if self.graph[*idx].fact_type == FactType::Module {
                        // Normalize names (- vs _)
                        let mod_name = self.graph[*idx].name.replace('-', "_");
                        let imp_name = import_path.replace('-', "_");

                        if imp_name.starts_with(&mod_name) {
                            self.graph.add_edge(source_idx, *idx, EdgeType::DependsOn);
                        }
                    }
                }
            }
        }
    }

    /// Detect circular dependencies using Tarjan's algorithm.
    ///
    /// Returns a list of strongly-connected components with more than one
    /// member, plus any self-loops.
    #[must_use]
    pub fn check_cycles(&self) -> Vec<Vec<String>> {
        let sccs = tarjan_scc(&self.graph);
        let mut cycles = Vec::new();
        for scc in sccs {
            if scc.len() > 1 {
                cycles.push(scc.iter().map(|idx| self.graph[*idx].id.clone()).collect());
            } else if scc.len() == 1 {
                // Check for self-loop
                if let Some(idx) = scc.first().copied()
                    && self.graph.contains_edge(idx, idx)
                {
                    cycles.push(vec![self.graph[idx].id.clone()]);
                }
            }
        }
        cycles
    }

    /// Check for forbidden dependencies between architectural layers.
    ///
    /// An edge from a node whose name contains `source_pattern` to a node
    /// whose name contains `target_pattern` via the `Contains → DependsOn`
    /// two-hop path is reported as a violation.
    #[must_use]
    pub fn check_layer_violation(
        &self,
        source_pattern: &str,
        target_pattern: &str,
    ) -> Vec<(String, String)> {
        let mut violations = Vec::new();

        // Find nodes matching source pattern
        let source_nodes: Vec<NodeIndex> = self
            .graph
            .node_indices()
            .filter(|idx| self.graph[*idx].name.contains(source_pattern))
            .collect();

        // Find nodes matching target pattern
        let target_nodes: Vec<NodeIndex> = self
            .graph
            .node_indices()
            .filter(|idx| self.graph[*idx].name.contains(target_pattern))
            .collect();

        for s_idx in &source_nodes {
            // Traverse: s_idx (Module/File) -> Contains -> Child (Import) -> DependsOn -> t_idx (Module)
            let children = self.graph.neighbors_directed(*s_idx, Direction::Outgoing);
            for child_idx in children {
                if let Some(edge) = self.graph.find_edge(*s_idx, child_idx)
                    && self.graph[edge] == EdgeType::Contains
                {
                    let dependencies = self
                        .graph
                        .neighbors_directed(child_idx, Direction::Outgoing);
                    for dep_idx in dependencies {
                        if let Some(dep_edge) = self.graph.find_edge(child_idx, dep_idx)
                            && self.graph[dep_edge] == EdgeType::DependsOn
                        {
                            // Check if dep_idx is in target_nodes
                            if target_nodes.contains(&dep_idx) {
                                violations.push((
                                    self.graph[*s_idx].name.clone(),
                                    self.graph[dep_idx].name.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        violations
    }
}
