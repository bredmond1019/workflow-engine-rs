//! Topological sorting algorithms for learning path ordering
//! 
//! Implements Kahn's algorithm and DFS-based topological sorting
//! to determine prerequisite ordering for concepts in learning paths.

use crate::graph::Concept;
use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Dependency relationship between concepts
#[derive(Debug, Clone)]
pub struct Dependency {
    pub concept_id: Uuid,
    pub prerequisite_id: Uuid,
    pub strength: DependencyStrength,
}

/// Strength of dependency relationship
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyStrength {
    Required,    // Must complete prerequisite
    Recommended, // Helpful but not required
    Optional,    // Minor benefit
}

/// Result of topological sorting
#[derive(Debug, Clone)]
pub struct TopologicalResult {
    pub ordered_concepts: Vec<Uuid>,
    pub concepts: Vec<Concept>,
    pub levels: Vec<Vec<Uuid>>, // Concepts grouped by level (can be learned in parallel)
}

/// Topological sorting implementation
#[derive(Clone)]
pub struct TopologicalSort {
    concepts: HashMap<Uuid, Concept>,
    dependencies: Vec<Dependency>,
}

impl TopologicalSort {
    /// Create a new topological sorter
    pub fn new() -> Self {
        Self {
            concepts: HashMap::new(),
            dependencies: Vec::new(),
        }
    }

    /// Add concepts to the graph
    pub fn add_concepts(&mut self, concepts: Vec<Concept>) {
        for concept in concepts {
            self.concepts.insert(concept.id, concept);
        }
    }

    /// Add dependencies between concepts
    pub fn add_dependencies(&mut self, dependencies: Vec<Dependency>) {
        self.dependencies.extend(dependencies);
    }

    /// Sort concepts using Kahn's algorithm
    pub async fn kahn_sort(&self, concept_ids: Option<Vec<Uuid>>) -> Result<TopologicalResult> {
        info!("Starting Kahn's topological sort");
        
        let node_set: HashSet<Uuid> = if let Some(ids) = concept_ids {
            ids.into_iter().collect()
        } else {
            self.concepts.keys().cloned().collect()
        };

        // Build adjacency list and in-degree count
        let mut adj_list: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();

        // Initialize all nodes
        for &concept_id in &node_set {
            adj_list.entry(concept_id).or_default();
            in_degree.entry(concept_id).or_insert(0);
        }

        // Build graph from dependencies
        for dep in &self.dependencies {
            if node_set.contains(&dep.concept_id) && node_set.contains(&dep.prerequisite_id) {
                // Only consider required and recommended dependencies for ordering
                if matches!(dep.strength, DependencyStrength::Required | DependencyStrength::Recommended) {
                    adj_list.entry(dep.prerequisite_id).or_default().push(dep.concept_id);
                    *in_degree.entry(dep.concept_id).or_insert(0) += 1;
                }
            }
        }

        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        let mut levels = Vec::new();
        let mut current_level = Vec::new();

        // Find all nodes with no incoming edges
        for (&concept_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back((concept_id, 0)); // (concept_id, level)
                current_level.push(concept_id);
            }
        }

        if !current_level.is_empty() {
            levels.push(current_level.clone());
            current_level.clear();
        }

        let mut current_level_num = 0;

        while let Some((concept_id, level)) = queue.pop_front() {
            // If we've moved to a new level, start a new level group
            if level > current_level_num {
                if !current_level.is_empty() {
                    levels.push(current_level.clone());
                    current_level.clear();
                }
                current_level_num = level;
            }

            result.push(concept_id);

            // Reduce in-degree for all neighbors
            if let Some(neighbors) = adj_list.get(&concept_id) {
                for &neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(&neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back((neighbor, level + 1));
                            current_level.push(neighbor);
                        }
                    }
                }
            }
        }

        // Add the last level if it has concepts
        if !current_level.is_empty() {
            levels.push(current_level);
        }

        // Check for cycles
        if result.len() != node_set.len() {
            let remaining: Vec<Uuid> = node_set.into_iter()
                .filter(|id| !result.contains(id))
                .collect();
            warn!("Cycle detected in dependency graph, remaining nodes: {:?}", remaining);
            return Err(anyhow!("Cycle detected in dependency graph"));
        }

        let concepts = self.get_concepts_for_ids(&result)?;

        debug!("Kahn's sort completed: {} concepts, {} levels", result.len(), levels.len());

        Ok(TopologicalResult {
            ordered_concepts: result,
            concepts,
            levels,
        })
    }

    /// Sort concepts using DFS-based approach
    pub async fn dfs_sort(&self, concept_ids: Option<Vec<Uuid>>) -> Result<TopologicalResult> {
        info!("Starting DFS topological sort");
        
        let node_set: HashSet<Uuid> = if let Some(ids) = concept_ids {
            ids.into_iter().collect()
        } else {
            self.concepts.keys().cloned().collect()
        };

        // Build adjacency list
        let mut adj_list: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for &concept_id in &node_set {
            adj_list.entry(concept_id).or_default();
        }

        for dep in &self.dependencies {
            if node_set.contains(&dep.concept_id) && node_set.contains(&dep.prerequisite_id) {
                if matches!(dep.strength, DependencyStrength::Required | DependencyStrength::Recommended) {
                    adj_list.entry(dep.prerequisite_id).or_default().push(dep.concept_id);
                }
            }
        }

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut result = Vec::new();

        // DFS visit all nodes
        for &concept_id in &node_set {
            if !visited.contains(&concept_id) {
                self.dfs_visit(concept_id, &adj_list, &mut visited, &mut rec_stack, &mut result)?;
            }
        }

        result.reverse(); // DFS gives reverse topological order

        // Calculate levels for DFS result
        let levels = self.calculate_levels(&result, &adj_list)?;
        let concepts = self.get_concepts_for_ids(&result)?;

        debug!("DFS sort completed: {} concepts, {} levels", result.len(), levels.len());

        Ok(TopologicalResult {
            ordered_concepts: result,
            concepts,
            levels,
        })
    }

    /// Sort with priority for specific concept categories
    pub async fn priority_sort(
        &self,
        concept_ids: Option<Vec<Uuid>>,
        priority_categories: Vec<String>,
    ) -> Result<TopologicalResult> {
        info!("Starting priority topological sort with categories: {:?}", priority_categories);
        
        // First get basic topological order
        let mut basic_result = self.kahn_sort(concept_ids).await?;
        
        // Group concepts by level and then reorder within each level by priority
        let mut reordered_levels = Vec::new();
        
        for level in basic_result.levels {
            let mut level_concepts: Vec<(Uuid, usize)> = level.into_iter()
                .map(|id| {
                    let priority = if let Some(concept) = self.concepts.get(&id) {
                        priority_categories.iter()
                            .position(|cat| cat == &concept.category)
                            .unwrap_or(priority_categories.len())
                    } else {
                        priority_categories.len()
                    };
                    (id, priority)
                })
                .collect();
            
            // Sort by priority (lower index = higher priority)
            level_concepts.sort_by_key(|(_, priority)| *priority);
            
            let reordered_level: Vec<Uuid> = level_concepts.into_iter()
                .map(|(id, _)| id)
                .collect();
            
            reordered_levels.push(reordered_level);
        }

        // Flatten the reordered levels to get final order
        let ordered_concepts: Vec<Uuid> = reordered_levels.iter()
            .flatten()
            .cloned()
            .collect();

        let concepts = self.get_concepts_for_ids(&ordered_concepts)?;

        basic_result.ordered_concepts = ordered_concepts;
        basic_result.concepts = concepts;
        basic_result.levels = reordered_levels;

        debug!("Priority sort completed with {} priority categories", priority_categories.len());

        Ok(basic_result)
    }

    /// Find strongly connected components (cycles) in the dependency graph
    pub async fn find_cycles(&self) -> Result<Vec<Vec<Uuid>>> {
        info!("Finding cycles in dependency graph");
        
        let node_set: HashSet<Uuid> = self.concepts.keys().cloned().collect();
        
        // Build adjacency list
        let mut adj_list: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for &concept_id in &node_set {
            adj_list.entry(concept_id).or_default();
        }

        for dep in &self.dependencies {
            if node_set.contains(&dep.concept_id) && node_set.contains(&dep.prerequisite_id) {
                adj_list.entry(dep.prerequisite_id).or_default().push(dep.concept_id);
            }
        }

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut cycles = Vec::new();

        for &concept_id in &node_set {
            if !visited.contains(&concept_id) {
                let mut current_path = Vec::new();
                self.find_cycles_dfs(
                    concept_id,
                    &adj_list,
                    &mut visited,
                    &mut rec_stack,
                    &mut current_path,
                    &mut cycles,
                )?;
            }
        }

        debug!("Found {} cycles in dependency graph", cycles.len());
        Ok(cycles)
    }

    /// Validate that a proposed learning path respects dependencies
    pub fn validate_learning_path(&self, path: &[Uuid]) -> Result<Vec<String>> {
        let mut violations = Vec::new();
        let mut completed = HashSet::new();

        for (index, &concept_id) in path.iter().enumerate() {
            // Check if all required prerequisites are completed
            for dep in &self.dependencies {
                if dep.concept_id == concept_id && dep.strength == DependencyStrength::Required {
                    if !completed.contains(&dep.prerequisite_id) {
                        if let (Some(concept), Some(prereq)) = (
                            self.concepts.get(&concept_id),
                            self.concepts.get(&dep.prerequisite_id)
                        ) {
                            violations.push(format!(
                                "Concept '{}' at position {} requires '{}' which appears later or is missing",
                                concept.name, index, prereq.name
                            ));
                        }
                    }
                }
            }
            
            completed.insert(concept_id);
        }

        Ok(violations)
    }

    /// DFS visit helper for DFS-based topological sort
    fn dfs_visit(
        &self,
        concept_id: Uuid,
        adj_list: &HashMap<Uuid, Vec<Uuid>>,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
        result: &mut Vec<Uuid>,
    ) -> Result<()> {
        visited.insert(concept_id);
        rec_stack.insert(concept_id);

        if let Some(neighbors) = adj_list.get(&concept_id) {
            for &neighbor in neighbors {
                if rec_stack.contains(&neighbor) {
                    return Err(anyhow!("Cycle detected involving concept {:?}", neighbor));
                }
                if !visited.contains(&neighbor) {
                    self.dfs_visit(neighbor, adj_list, visited, rec_stack, result)?;
                }
            }
        }

        rec_stack.remove(&concept_id);
        result.push(concept_id);
        Ok(())
    }

    /// Calculate levels for concepts based on their dependencies
    fn calculate_levels(&self, ordered_concepts: &[Uuid], _adj_list: &HashMap<Uuid, Vec<Uuid>>) -> Result<Vec<Vec<Uuid>>> {
        let mut levels = Vec::new();
        let mut concept_levels = HashMap::new();

        // Assign levels based on longest path from roots
        for &concept_id in ordered_concepts {
            let mut max_prereq_level = -1i32;
            
            // Find all prerequisites of this concept
            for dep in &self.dependencies {
                if dep.concept_id == concept_id {
                    if let Some(&prereq_level) = concept_levels.get(&dep.prerequisite_id) {
                        max_prereq_level = max_prereq_level.max(prereq_level);
                    }
                }
            }
            
            let concept_level = (max_prereq_level + 1) as usize;
            concept_levels.insert(concept_id, concept_level as i32);
            
            // Ensure we have enough levels
            while levels.len() <= concept_level {
                levels.push(Vec::new());
            }
            
            levels[concept_level].push(concept_id);
        }

        Ok(levels)
    }

    /// DFS for cycle detection
    fn find_cycles_dfs(
        &self,
        concept_id: Uuid,
        adj_list: &HashMap<Uuid, Vec<Uuid>>,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
        current_path: &mut Vec<Uuid>,
        cycles: &mut Vec<Vec<Uuid>>,
    ) -> Result<()> {
        visited.insert(concept_id);
        rec_stack.insert(concept_id);
        current_path.push(concept_id);

        if let Some(neighbors) = adj_list.get(&concept_id) {
            for &neighbor in neighbors {
                if rec_stack.contains(&neighbor) {
                    // Found a cycle - extract it from current_path
                    if let Some(cycle_start) = current_path.iter().position(|&id| id == neighbor) {
                        let cycle = current_path[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                } else if !visited.contains(&neighbor) {
                    self.find_cycles_dfs(neighbor, adj_list, visited, rec_stack, current_path, cycles)?;
                }
            }
        }

        rec_stack.remove(&concept_id);
        current_path.pop();
        Ok(())
    }

    /// Get concepts for given IDs
    fn get_concepts_for_ids(&self, ids: &[Uuid]) -> Result<Vec<Concept>> {
        ids.iter()
            .map(|&id| {
                self.concepts.get(&id)
                    .cloned()
                    .ok_or_else(|| anyhow!("Concept not found: {:?}", id))
            })
            .collect()
    }
}

impl Default for TopologicalSort {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_concept(name: &str, category: &str) -> Concept {
        Concept {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: Some(format!("Test concept: {}", name)),
            difficulty: "intermediate".to_string(),
            category: category.to_string(),
            subcategory: None,
            tags: vec![],
            quality_score: 0.8,
            estimated_time: Some(60.0),
            embeddings: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
        }
    }

    #[tokio::test]
    async fn test_kahn_simple_ordering() {
        let mut topo_sort = TopologicalSort::new();
        
        let concept1 = create_test_concept("Basic Math", "math");
        let concept2 = create_test_concept("Algebra", "math");
        let concept3 = create_test_concept("Calculus", "math");
        
        let id1 = concept1.id;
        let id2 = concept2.id;
        let id3 = concept3.id;
        
        topo_sort.add_concepts(vec![concept1, concept2, concept3]);
        topo_sort.add_dependencies(vec![
            Dependency {
                concept_id: id2,
                prerequisite_id: id1,
                strength: DependencyStrength::Required,
            },
            Dependency {
                concept_id: id3,
                prerequisite_id: id2,
                strength: DependencyStrength::Required,
            },
        ]);

        let result = topo_sort.kahn_sort(None).await.unwrap();
        
        assert_eq!(result.ordered_concepts, vec![id1, id2, id3]);
        assert_eq!(result.levels.len(), 3);
        assert_eq!(result.levels[0], vec![id1]);
        assert_eq!(result.levels[1], vec![id2]);
        assert_eq!(result.levels[2], vec![id3]);
    }

    #[tokio::test]
    async fn test_parallel_learning() {
        let mut topo_sort = TopologicalSort::new();
        
        let concept1 = create_test_concept("Foundation", "math");
        let concept2 = create_test_concept("Branch A", "math");
        let concept3 = create_test_concept("Branch B", "math");
        let concept4 = create_test_concept("Advanced", "math");
        
        let id1 = concept1.id;
        let id2 = concept2.id;
        let id3 = concept3.id;
        let id4 = concept4.id;
        
        topo_sort.add_concepts(vec![concept1, concept2, concept3, concept4]);
        topo_sort.add_dependencies(vec![
            Dependency { concept_id: id2, prerequisite_id: id1, strength: DependencyStrength::Required },
            Dependency { concept_id: id3, prerequisite_id: id1, strength: DependencyStrength::Required },
            Dependency { concept_id: id4, prerequisite_id: id2, strength: DependencyStrength::Required },
            Dependency { concept_id: id4, prerequisite_id: id3, strength: DependencyStrength::Required },
        ]);

        let result = topo_sort.kahn_sort(None).await.unwrap();
        
        // Foundation should be first
        assert_eq!(result.levels[0], vec![id1]);
        
        // Branch A and B should be in the same level (can be learned in parallel)
        assert_eq!(result.levels[1].len(), 2);
        assert!(result.levels[1].contains(&id2));
        assert!(result.levels[1].contains(&id3));
        
        // Advanced should be last
        assert_eq!(result.levels[2], vec![id4]);
    }

    #[tokio::test]
    async fn test_priority_sorting() {
        let mut topo_sort = TopologicalSort::new();
        
        let concept1 = create_test_concept("Math Concept", "math");
        let concept2 = create_test_concept("Science Concept", "science");
        let concept3 = create_test_concept("Programming Concept", "programming");
        
        let id1 = concept1.id;
        let id2 = concept2.id;
        let id3 = concept3.id;
        
        topo_sort.add_concepts(vec![concept1, concept2, concept3]);

        let priority_categories = vec!["programming".to_string(), "math".to_string(), "science".to_string()];
        let result = topo_sort.priority_sort(None, priority_categories).await.unwrap();
        
        // Programming should come first due to priority
        assert_eq!(result.ordered_concepts[0], id3);
    }

    #[tokio::test]
    async fn test_cycle_detection() {
        let mut topo_sort = TopologicalSort::new();
        
        let concept1 = create_test_concept("Concept A", "math");
        let concept2 = create_test_concept("Concept B", "math");
        
        let id1 = concept1.id;
        let id2 = concept2.id;
        
        topo_sort.add_concepts(vec![concept1, concept2]);
        topo_sort.add_dependencies(vec![
            Dependency { concept_id: id2, prerequisite_id: id1, strength: DependencyStrength::Required },
            Dependency { concept_id: id1, prerequisite_id: id2, strength: DependencyStrength::Required },
        ]);

        let result = topo_sort.kahn_sort(None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_path_validation() {
        let mut topo_sort = TopologicalSort::new();
        
        let concept1 = create_test_concept("Basic", "math");
        let concept2 = create_test_concept("Advanced", "math");
        
        let id1 = concept1.id;
        let id2 = concept2.id;
        
        topo_sort.add_concepts(vec![concept1, concept2]);
        topo_sort.add_dependencies(vec![
            Dependency { concept_id: id2, prerequisite_id: id1, strength: DependencyStrength::Required },
        ]);

        // Valid path
        let violations = topo_sort.validate_learning_path(&[id1, id2]).unwrap();
        assert!(violations.is_empty());

        // Invalid path (prerequisite missing)
        let violations = topo_sort.validate_learning_path(&[id2]).unwrap();
        assert!(!violations.is_empty());
    }
}