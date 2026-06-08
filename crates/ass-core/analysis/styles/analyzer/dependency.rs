//! Style dependency ordering and circular-inheritance detection.
//!
//! Builds a topological ordering of styles based on their parent references and
//! detects circular inheritance via depth-first search before resolution.

use super::{AnalysisOptions, StyleAnalyzer};
use crate::{
    analysis::styles::validation::{StyleConflict, StyleInheritance},
    parser::Style,
};
use alloc::{collections::BTreeMap, collections::BTreeSet, vec::Vec};

impl<'a> StyleAnalyzer<'a> {
    /// Build dependency order for styles using topological sort
    /// Returns None if circular dependency is detected
    pub(super) fn build_dependency_order(
        &mut self,
        styles: &'a [Style<'a>],
    ) -> Option<Vec<&'a Style<'a>>> {
        // Create style map for quick lookup
        let style_map: BTreeMap<&str, &Style> = styles.iter().map(|s| (s.name, s)).collect();

        // Build adjacency list (child -> parent)
        let mut dependencies: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        let mut in_degree: BTreeMap<&str, usize> = BTreeMap::new();

        // Initialize all styles
        for style in styles {
            dependencies.insert(style.name, BTreeSet::new());
            in_degree.insert(style.name, 0);
        }

        // Build dependency graph
        for style in styles {
            if let Some(parent_name) = style.parent {
                if style_map.contains_key(parent_name) {
                    dependencies
                        .get_mut(style.name)
                        .unwrap()
                        .insert(parent_name);
                    *in_degree.get_mut(parent_name).unwrap() += 1;

                    // Track inheritance for analysis
                    if self.config.options.contains(AnalysisOptions::INHERITANCE) {
                        if let Some(inheritance) = self.inheritance_info.get_mut(style.name) {
                            inheritance.set_parent(parent_name);
                        } else {
                            let mut inheritance = StyleInheritance::new(style.name);
                            inheritance.set_parent(parent_name);
                            self.inheritance_info.insert(style.name, inheritance);
                        }
                    }
                } else {
                    // Parent style not found - add warning conflict
                    self.conflicts
                        .push(StyleConflict::missing_parent(style.name, parent_name));
                }
            } else if self.config.options.contains(AnalysisOptions::INHERITANCE) {
                // Style has no parent
                self.inheritance_info
                    .insert(style.name, StyleInheritance::new(style.name));
            }
        }

        // Check for circular dependencies using DFS
        if Self::has_circular_dependency(&dependencies) {
            self.conflicts.push(StyleConflict::circular_inheritance(
                dependencies.keys().copied().collect(),
            ));
            return None;
        }

        // Perform topological sort
        let mut result = Vec::new();
        let mut queue: Vec<&str> = Vec::new();

        // Find all nodes with no dependencies
        for (name, degree) in &in_degree {
            if *degree == 0 {
                queue.push(name);
            }
        }

        while let Some(current) = queue.pop() {
            if let Some(style) = style_map.get(current) {
                result.push(*style);
            }

            // Update in-degrees
            for (child, parents) in &dependencies {
                if parents.contains(current) {
                    if let Some(degree) = in_degree.get_mut(child) {
                        *degree = degree.saturating_sub(1);
                        if *degree == 0 {
                            queue.push(child);
                        }
                    }
                }
            }
        }

        // Check if all styles were processed
        if result.len() == styles.len() {
            Some(result)
        } else {
            // Not all styles processed - circular dependency exists
            None
        }
    }

    /// Check for circular dependencies using DFS
    fn has_circular_dependency(dependencies: &BTreeMap<&str, BTreeSet<&str>>) -> bool {
        let mut visited = BTreeSet::new();
        let mut rec_stack = BTreeSet::new();

        for node in dependencies.keys() {
            if !visited.contains(node)
                && Self::dfs_has_cycle(node, dependencies, &mut visited, &mut rec_stack)
            {
                return true;
            }
        }

        false
    }

    /// DFS helper for cycle detection
    fn dfs_has_cycle<'b>(
        node: &'b str,
        dependencies: &BTreeMap<&'b str, BTreeSet<&'b str>>,
        visited: &mut BTreeSet<&'b str>,
        rec_stack: &mut BTreeSet<&'b str>,
    ) -> bool {
        visited.insert(node);
        rec_stack.insert(node);

        if let Some(neighbors) = dependencies.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if Self::dfs_has_cycle(neighbor, dependencies, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }
}
