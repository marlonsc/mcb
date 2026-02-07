use crate::extractor::{Fact, FactType};
use petgraph::Direction;
use petgraph::algo::tarjan_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeType {
    DependsOn,
    Contains,
}

pub struct DependencyGraph {
    graph: DiGraph<Fact, EdgeType>,
    node_map: HashMap<String, NodeIndex>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

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
            let source_idx = *self.node_map.get(&fact.id).unwrap();

            // Add "Contains" edges (Parent -> Child)
            if let Some(parent_id) = &fact.parent_id {
                if let Some(parent_idx) = self.node_map.get(parent_id) {
                    self.graph
                        .add_edge(*parent_idx, source_idx, EdgeType::Contains);
                }
            }

            // Add "DependsOn" edges (Import -> Target Module)
            if fact.fact_type == FactType::Import {
                // Heuristic: simplistic resolution.
                // Using "name" as the target module/item name.
                // In a real system, we'd need a resolver.
                // Here we check if the imported name matches any known module ID.
                // Or if it matches "module::{name}".

                // Try to find a node that matches the import name
                // This is a simplification. A real resolver is complex.
                // For "use crate::foo::bar", name is "crate::foo::bar"
                // We look for facts with ID "module::foo::bar" or similar.

                // For now, we iterate all nodes to find partial matches (very slow but functional for proof of concept)
                // Or we rely on the fact that we might have extracted "module::x"

                let import_path = &fact.name;

                // Simple exact match check first
                if let Some(target_idx) = self.node_map.get(import_path) {
                    self.graph
                        .add_edge(source_idx, *target_idx, EdgeType::DependsOn);
                }

                // Check for module prefix (e.g. import "mcb_domain::..." depends on module "mcb_domain")
                // This is crucial for layer validation.
                for (_id, idx) in &self.node_map {
                    // If the import path starts with the module name (e.g. mcb_domain)
                    // and the node is a Module.
                    if self.graph[*idx].fact_type == FactType::Module {
                        // Heuristic: check if import path starts with module name
                        // We need to handle "crate::" or raw names.

                        // Example: id = "module::mcb-domain", name="mcb-domain"
                        // Import = "mcb_domain::params"

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

    pub fn check_cycles(&self) -> Vec<Vec<String>> {
        let sccs = tarjan_scc(&self.graph);
        let mut cycles = Vec::new();
        for scc in sccs {
            if scc.len() > 1 {
                cycles.push(scc.iter().map(|idx| self.graph[*idx].id.clone()).collect());
            } else if scc.len() == 1 {
                // Check for self-loop
                let idx = scc[0];
                if self.graph.contains_edge(idx, idx) {
                    cycles.push(vec![self.graph[idx].id.clone()]);
                }
            }
        }
        cycles
    }

    // Check if source_layer depends on target_layer
    // Layers are defined by module name patterns.
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
            // BFS or direct neighbor check?
            // "DependsOn" is likely direct from an Import node to a Module node.
            // But the Import node is contained in a Module.
            // So: Module A (contains) -> Import X (depends on) -> Module B.
            // We want to know if Module A depends on Module B.

            // Traverse: s_idx (Module/File) -> Contains -> Child (Import) -> DependsOn -> t_idx (Module)

            let children = self.graph.neighbors_directed(*s_idx, Direction::Outgoing);
            for child_idx in children {
                let edge = self.graph.find_edge(*s_idx, child_idx).unwrap();
                if self.graph[edge] == EdgeType::Contains {
                    let dependencies = self
                        .graph
                        .neighbors_directed(child_idx, Direction::Outgoing);
                    for dep_idx in dependencies {
                        let dep_edge = self.graph.find_edge(child_idx, dep_idx).unwrap();
                        if self.graph[dep_edge] == EdgeType::DependsOn {
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
