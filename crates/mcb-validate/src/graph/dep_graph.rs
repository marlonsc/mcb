//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Dependency graph construction and layer-boundary analysis.
//!
//! Builds a `petgraph` directed graph from extracted [`Fact`]s, enabling
//! cycle detection and inter-layer dependency checks.

use crate::extractor::{Fact, FactType};
use petgraph::Direction;
use petgraph::algo::tarjan_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::{HashMap, HashSet};

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
        self.add_nodes(facts);
        let module_indices = self.module_indices();
        for fact in facts {
            self.add_edges_for_fact(fact, &module_indices);
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
        let source_nodes: Vec<NodeIndex> = self
            .graph
            .node_indices()
            .filter(|idx| self.graph[*idx].name.contains(source_pattern))
            .collect();
        let target_nodes: HashSet<NodeIndex> = self
            .graph
            .node_indices()
            .filter(|idx| self.graph[*idx].name.contains(target_pattern))
            .collect();

        source_nodes
            .iter()
            .flat_map(|source| self.violations_for_source(*source, &target_nodes))
            .collect()
    }

    fn add_nodes(&mut self, facts: &[Fact]) {
        for fact in facts {
            if self.node_map.contains_key(&fact.id) {
                continue;
            }

            let idx = self.graph.add_node(fact.clone());
            self.node_map.insert(fact.id.clone(), idx);
        }
    }

    fn module_indices(&self) -> Vec<NodeIndex> {
        self.node_map
            .values()
            .copied()
            .filter(|idx| self.graph[*idx].fact_type == FactType::Module)
            .collect()
    }

    fn add_edges_for_fact(&mut self, fact: &Fact, module_indices: &[NodeIndex]) {
        let Some(source_idx) = self.node_map.get(&fact.id).copied() else {
            return;
        };

        self.add_contains_edge(source_idx, fact);
        self.add_import_dep_edges(source_idx, fact, module_indices);
    }

    fn add_contains_edge(&mut self, source_idx: NodeIndex, fact: &Fact) {
        let Some(parent_id) = fact.parent_id.as_deref() else {
            return;
        };
        let Some(parent_idx) = self.node_map.get(parent_id).copied() else {
            return;
        };

        self.graph
            .add_edge(parent_idx, source_idx, EdgeType::Contains);
    }

    fn add_import_dep_edges(
        &mut self,
        source_idx: NodeIndex,
        fact: &Fact,
        module_indices: &[NodeIndex],
    ) {
        if fact.fact_type != FactType::Import {
            return;
        }

        self.add_exact_import_dependency(source_idx, &fact.name);
        self.add_prefix_import_dependencies(source_idx, &fact.name, module_indices);
    }

    fn add_exact_import_dependency(&mut self, source_idx: NodeIndex, import_path: &str) {
        let Some(target_idx) = self.node_map.get(import_path).copied() else {
            return;
        };

        self.graph
            .add_edge(source_idx, target_idx, EdgeType::DependsOn);
    }

    fn add_prefix_import_dependencies(
        &mut self,
        source_idx: NodeIndex,
        import_path: &str,
        module_indices: &[NodeIndex],
    ) {
        let normalized_import = import_path.replace('-', "_");
        for idx in module_indices {
            let normalized_module = self.graph[*idx].name.replace('-', "_");
            if normalized_import.starts_with(&normalized_module) {
                self.graph.add_edge(source_idx, *idx, EdgeType::DependsOn);
            }
        }
    }

    fn violations_for_source(
        &self,
        source_idx: NodeIndex,
        target_nodes: &HashSet<NodeIndex>,
    ) -> Vec<(String, String)> {
        self.contains_neighbors(source_idx)
            .flat_map(|child_idx| self.depends_on_neighbors(child_idx))
            .filter(|dep_idx| target_nodes.contains(dep_idx))
            .map(|dep_idx| {
                (
                    self.graph[source_idx].name.clone(),
                    self.graph[dep_idx].name.clone(),
                )
            })
            .collect()
    }

    fn contains_neighbors(&self, source_idx: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.graph
            .neighbors_directed(source_idx, Direction::Outgoing)
            .filter(move |neighbor_idx| {
                self.graph
                    .find_edge(source_idx, *neighbor_idx)
                    .is_some_and(|edge| self.graph[edge] == EdgeType::Contains)
            })
    }

    fn depends_on_neighbors(&self, source_idx: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.graph
            .neighbors_directed(source_idx, Direction::Outgoing)
            .filter(move |neighbor_idx| {
                self.graph
                    .find_edge(source_idx, *neighbor_idx)
                    .is_some_and(|edge| self.graph[edge] == EdgeType::DependsOn)
            })
    }
}
