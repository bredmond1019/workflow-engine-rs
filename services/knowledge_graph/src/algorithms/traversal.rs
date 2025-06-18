//! Graph traversal algorithms for knowledge exploration
//! 
//! Implements BFS, DFS, and depth-limited search for exploring
//! concept relationships and finding related learning materials.

use crate::graph::Concept;
use anyhow::Result;
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, info};
use uuid::Uuid;

/// Result of a graph traversal
#[derive(Debug, Clone)]
pub struct TraversalResult {
    pub visited_concepts: Vec<Uuid>,
    pub concepts: Vec<Concept>,
    pub path_tree: HashMap<Uuid, Vec<Uuid>>, // Parent -> Children mapping
    pub distances: HashMap<Uuid, usize>,     // Distance from start node
}

/// Configuration for traversal algorithms
#[derive(Debug, Clone)]
pub struct TraversalConfig {
    pub max_depth: Option<usize>,
    pub max_nodes: Option<usize>,
    pub include_categories: Option<Vec<String>>,
    pub exclude_categories: Option<Vec<String>>,
    pub min_quality_score: Option<f32>,
    pub follow_relationships: Vec<RelationshipType>,
}

/// Types of relationships to follow during traversal
#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipType {
    Prerequisite,
    Progression,
    Similarity,
    Related,
    Inverse, // Follow edges in reverse direction
}

/// Edge between concepts with relationship information
#[derive(Debug, Clone)]
pub struct ConceptEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub relationship: RelationshipType,
    pub weight: f32,
}

impl Default for TraversalConfig {
    fn default() -> Self {
        Self {
            max_depth: Some(6),
            max_nodes: Some(100),
            include_categories: None,
            exclude_categories: None,
            min_quality_score: Some(0.5),
            follow_relationships: vec![
                RelationshipType::Prerequisite,
                RelationshipType::Progression,
                RelationshipType::Related,
            ],
        }
    }
}

/// Graph traversal implementation
#[derive(Clone)]
pub struct GraphTraversal {
    concepts: HashMap<Uuid, Concept>,
    edges: Vec<ConceptEdge>,
}

impl GraphTraversal {
    /// Create a new graph traversal instance
    pub fn new() -> Self {
        Self {
            concepts: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Add concepts to the graph
    pub fn add_concepts(&mut self, concepts: Vec<Concept>) {
        for concept in concepts {
            self.concepts.insert(concept.id, concept);
        }
    }

    /// Add edges between concepts
    pub fn add_edges(&mut self, edges: Vec<ConceptEdge>) {
        self.edges.extend(edges);
    }

    /// Breadth-First Search traversal
    pub async fn bfs_traversal(
        &self,
        start: Uuid,
        config: TraversalConfig,
    ) -> Result<TraversalResult> {
        info!("Starting BFS traversal from {:?}", start);
        
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut result_concepts = Vec::new();
        let mut path_tree = HashMap::new();
        let mut distances = HashMap::new();

        // Initialize with start node
        queue.push_back((start, 0));
        visited.insert(start);
        distances.insert(start, 0);

        while let Some((current_id, depth)) = queue.pop_front() {
            // Check depth limit
            if let Some(max_depth) = config.max_depth {
                if depth >= max_depth {
                    continue;
                }
            }

            // Check node limit
            if let Some(max_nodes) = config.max_nodes {
                if result_concepts.len() >= max_nodes {
                    break;
                }
            }

            // Add current concept to results if it passes filters
            if let Some(concept) = self.concepts.get(&current_id) {
                if self.passes_filters(concept, &config) {
                    result_concepts.push(current_id);
                }
            }

            // Explore neighbors
            let neighbors = self.get_neighbors(current_id, &config);
            let mut children = Vec::new();

            for neighbor_id in neighbors {
                if !visited.contains(&neighbor_id) {
                    visited.insert(neighbor_id);
                    distances.insert(neighbor_id, depth + 1);
                    queue.push_back((neighbor_id, depth + 1));
                    children.push(neighbor_id);
                }
            }

            if !children.is_empty() {
                path_tree.insert(current_id, children);
            }
        }

        let concepts = self.get_concepts_for_ids(&result_concepts)?;

        debug!("BFS completed: visited {} concepts, max depth {}", 
               result_concepts.len(), distances.values().max().unwrap_or(&0));

        Ok(TraversalResult {
            visited_concepts: result_concepts,
            concepts,
            path_tree,
            distances,
        })
    }

    /// Depth-First Search traversal
    pub async fn dfs_traversal(
        &self,
        start: Uuid,
        config: TraversalConfig,
    ) -> Result<TraversalResult> {
        info!("Starting DFS traversal from {:?}", start);
        
        let mut visited = HashSet::new();
        let mut result_concepts = Vec::new();
        let mut path_tree = HashMap::new();
        let mut distances = HashMap::new();

        self.dfs_visit(
            start,
            0,
            &config,
            &mut visited,
            &mut result_concepts,
            &mut path_tree,
            &mut distances,
        )?;

        let concepts = self.get_concepts_for_ids(&result_concepts)?;

        debug!("DFS completed: visited {} concepts, max depth {}", 
               result_concepts.len(), distances.values().max().unwrap_or(&0));

        Ok(TraversalResult {
            visited_concepts: result_concepts,
            concepts,
            path_tree,
            distances,
        })
    }

    /// Depth-limited search
    pub async fn depth_limited_search(
        &self,
        start: Uuid,
        goal: Uuid,
        max_depth: usize,
        config: TraversalConfig,
    ) -> Result<Option<Vec<Uuid>>> {
        info!("Starting depth-limited search from {:?} to {:?}, max depth: {}", 
              start, goal, max_depth);
        
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        if self.dfs_limited(start, goal, max_depth, 0, &config, &mut visited, &mut path)? {
            debug!("Found path of length {} within depth limit", path.len());
            Ok(Some(path))
        } else {
            debug!("No path found within depth limit");
            Ok(None)
        }
    }

    /// Iterative deepening search
    pub async fn iterative_deepening_search(
        &self,
        start: Uuid,
        goal: Uuid,
        max_depth: usize,
        config: TraversalConfig,
    ) -> Result<Option<Vec<Uuid>>> {
        info!("Starting iterative deepening search from {:?} to {:?}", start, goal);
        
        for depth in 1..=max_depth {
            debug!("Trying depth limit: {}", depth);
            
            if let Some(path) = self.depth_limited_search(start, goal, depth, config.clone()).await? {
                info!("Found path at depth {}: {} concepts", depth, path.len());
                return Ok(Some(path));
            }
        }

        debug!("No path found within maximum depth {}", max_depth);
        Ok(None)
    }

    /// Find all concepts within a certain distance
    pub async fn find_concepts_within_distance(
        &self,
        start: Uuid,
        max_distance: usize,
        config: TraversalConfig,
    ) -> Result<HashMap<usize, Vec<Uuid>>> {
        info!("Finding concepts within distance {} from {:?}", max_distance, start);
        
        let traversal_result = self.bfs_traversal(start, config).await?;
        let mut distance_groups = HashMap::new();

        for (concept_id, &distance) in &traversal_result.distances {
            if distance <= max_distance {
                distance_groups.entry(distance)
                    .or_insert_with(Vec::new)
                    .push(*concept_id);
            }
        }

        debug!("Found concepts at {} different distances", distance_groups.len());
        Ok(distance_groups)
    }

    /// Find strongly connected components (for concept clusters)
    pub async fn find_connected_components(&self) -> Result<Vec<Vec<Uuid>>> {
        info!("Finding connected components in the graph");
        
        let all_concepts: Vec<Uuid> = self.concepts.keys().cloned().collect();
        let mut visited = HashSet::new();
        let mut components = Vec::new();

        for &concept_id in &all_concepts {
            if !visited.contains(&concept_id) {
                let mut component = Vec::new();
                let config = TraversalConfig::default();
                self.dfs_component(concept_id, &config, &mut visited, &mut component)?;
                
                if !component.is_empty() {
                    components.push(component);
                }
            }
        }

        debug!("Found {} connected components", components.len());
        Ok(components)
    }

    /// Find concepts similar to a given concept
    pub async fn find_similar_concepts(
        &self,
        concept_id: Uuid,
        similarity_threshold: f32,
        max_results: usize,
    ) -> Result<Vec<(Uuid, f32)>> {
        info!("Finding concepts similar to {:?}", concept_id);
        
        let base_concept = self.concepts.get(&concept_id)
            .ok_or_else(|| anyhow::anyhow!("Concept not found: {:?}", concept_id))?;

        let mut similarities = Vec::new();

        for (&other_id, other_concept) in &self.concepts {
            if other_id != concept_id {
                let similarity = self.calculate_concept_similarity(base_concept, other_concept);
                if similarity >= similarity_threshold {
                    similarities.push((other_id, similarity));
                }
            }
        }

        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similarities.truncate(max_results);

        debug!("Found {} similar concepts above threshold {}", 
               similarities.len(), similarity_threshold);

        Ok(similarities)
    }

    /// DFS visit helper
    fn dfs_visit(
        &self,
        current_id: Uuid,
        depth: usize,
        config: &TraversalConfig,
        visited: &mut HashSet<Uuid>,
        result_concepts: &mut Vec<Uuid>,
        path_tree: &mut HashMap<Uuid, Vec<Uuid>>,
        distances: &mut HashMap<Uuid, usize>,
    ) -> Result<()> {
        // Check depth limit
        if let Some(max_depth) = config.max_depth {
            if depth >= max_depth {
                return Ok(());
            }
        }

        // Check node limit
        if let Some(max_nodes) = config.max_nodes {
            if result_concepts.len() >= max_nodes {
                return Ok(());
            }
        }

        visited.insert(current_id);
        distances.insert(current_id, depth);

        // Add current concept if it passes filters
        if let Some(concept) = self.concepts.get(&current_id) {
            if self.passes_filters(concept, config) {
                result_concepts.push(current_id);
            }
        }

        // Explore neighbors
        let neighbors = self.get_neighbors(current_id, config);
        let mut children = Vec::new();

        for neighbor_id in neighbors {
            if !visited.contains(&neighbor_id) {
                children.push(neighbor_id);
                self.dfs_visit(neighbor_id, depth + 1, config, visited, result_concepts, path_tree, distances)?;
            }
        }

        if !children.is_empty() {
            path_tree.insert(current_id, children);
        }

        Ok(())
    }

    /// DFS limited helper for depth-limited search
    fn dfs_limited(
        &self,
        current_id: Uuid,
        goal: Uuid,
        max_depth: usize,
        depth: usize,
        config: &TraversalConfig,
        visited: &mut HashSet<Uuid>,
        path: &mut Vec<Uuid>,
    ) -> Result<bool> {
        if current_id == goal {
            path.push(current_id);
            return Ok(true);
        }

        if depth >= max_depth {
            return Ok(false);
        }

        visited.insert(current_id);
        path.push(current_id);

        let neighbors = self.get_neighbors(current_id, config);
        for neighbor_id in neighbors {
            if !visited.contains(&neighbor_id) {
                if self.dfs_limited(neighbor_id, goal, max_depth, depth + 1, config, visited, path)? {
                    return Ok(true);
                }
            }
        }

        path.pop();
        visited.remove(&current_id);
        Ok(false)
    }

    /// DFS for finding connected components
    fn dfs_component(
        &self,
        current_id: Uuid,
        config: &TraversalConfig,
        visited: &mut HashSet<Uuid>,
        component: &mut Vec<Uuid>,
    ) -> Result<()> {
        visited.insert(current_id);
        component.push(current_id);

        let neighbors = self.get_neighbors(current_id, config);
        for neighbor_id in neighbors {
            if !visited.contains(&neighbor_id) {
                self.dfs_component(neighbor_id, config, visited, component)?;
            }
        }

        Ok(())
    }

    /// Get neighbors of a concept based on configuration
    fn get_neighbors(&self, concept_id: Uuid, config: &TraversalConfig) -> Vec<Uuid> {
        let mut neighbors = Vec::new();

        for edge in &self.edges {
            let should_follow = config.follow_relationships.contains(&edge.relationship);
            
            if should_follow {
                if edge.from == concept_id {
                    neighbors.push(edge.to);
                } else if edge.to == concept_id && config.follow_relationships.contains(&RelationshipType::Inverse) {
                    neighbors.push(edge.from);
                }
            }
        }

        neighbors
    }

    /// Check if a concept passes the configured filters
    fn passes_filters(&self, concept: &Concept, config: &TraversalConfig) -> bool {
        // Check quality score
        if let Some(min_quality) = config.min_quality_score {
            if concept.quality_score < min_quality {
                return false;
            }
        }

        // Check include categories
        if let Some(include_cats) = &config.include_categories {
            if !include_cats.contains(&concept.category) {
                return false;
            }
        }

        // Check exclude categories
        if let Some(exclude_cats) = &config.exclude_categories {
            if exclude_cats.contains(&concept.category) {
                return false;
            }
        }

        true
    }

    /// Calculate similarity between two concepts
    pub fn calculate_concept_similarity(&self, concept1: &Concept, concept2: &Concept) -> f32 {
        let mut similarity = 0.0;

        // Category similarity
        if concept1.category == concept2.category {
            similarity += 0.4;
        }

        // Subcategory similarity
        if concept1.subcategory == concept2.subcategory && concept1.subcategory.is_some() {
            similarity += 0.2;
        }

        // Tag overlap
        let common_tags = concept1.tags.iter()
            .filter(|tag| concept2.tags.contains(tag))
            .count();
        let total_tags = (concept1.tags.len() + concept2.tags.len()).max(1);
        similarity += 0.3 * (common_tags as f32 / total_tags as f32);

        // Difficulty similarity
        if concept1.difficulty == concept2.difficulty {
            similarity += 0.1;
        }

        similarity.min(1.0)
    }

    /// Get concepts for given IDs
    fn get_concepts_for_ids(&self, ids: &[Uuid]) -> Result<Vec<Concept>> {
        ids.iter()
            .map(|&id| {
                self.concepts.get(&id)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Concept not found: {:?}", id))
            })
            .collect()
    }
}

impl Default for GraphTraversal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_concept(name: &str, category: &str, tags: Vec<&str>) -> Concept {
        Concept {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: Some(format!("Test concept: {}", name)),
            difficulty: "intermediate".to_string(),
            category: category.to_string(),
            subcategory: None,
            tags: tags.into_iter().map(|s| s.to_string()).collect(),
            quality_score: 0.8,
            estimated_time: Some(60.0),
            embeddings: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
        }
    }

    #[tokio::test]
    async fn test_bfs_traversal() {
        let mut traversal = GraphTraversal::new();
        
        let concept1 = create_test_concept("Start", "math", vec!["algebra"]);
        let concept2 = create_test_concept("Middle", "math", vec!["algebra"]);
        let concept3 = create_test_concept("End", "math", vec!["algebra"]);
        
        let id1 = concept1.id;
        let id2 = concept2.id;
        let id3 = concept3.id;
        
        traversal.add_concepts(vec![concept1, concept2, concept3]);
        traversal.add_edges(vec![
            ConceptEdge { from: id1, to: id2, relationship: RelationshipType::Progression, weight: 1.0 },
            ConceptEdge { from: id2, to: id3, relationship: RelationshipType::Progression, weight: 1.0 },
        ]);

        let config = TraversalConfig::default();
        let result = traversal.bfs_traversal(id1, config).await.unwrap();
        
        assert_eq!(result.visited_concepts.len(), 3);
        assert_eq!(result.distances[&id1], 0);
        assert_eq!(result.distances[&id2], 1);
        assert_eq!(result.distances[&id3], 2);
    }

    #[tokio::test]
    async fn test_depth_limited_search() {
        let mut traversal = GraphTraversal::new();
        
        let concept1 = create_test_concept("Start", "math", vec![]);
        let concept2 = create_test_concept("Middle", "math", vec![]);
        let concept3 = create_test_concept("Goal", "math", vec![]);
        
        let id1 = concept1.id;
        let id2 = concept2.id;
        let id3 = concept3.id;
        
        traversal.add_concepts(vec![concept1, concept2, concept3]);
        traversal.add_edges(vec![
            ConceptEdge { from: id1, to: id2, relationship: RelationshipType::Related, weight: 1.0 },
            ConceptEdge { from: id2, to: id3, relationship: RelationshipType::Related, weight: 1.0 },
        ]);

        let config = TraversalConfig::default();
        
        // Should find path with depth limit 3
        let result = traversal.depth_limited_search(id1, id3, 3, config.clone()).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec![id1, id2, id3]);
        
        // Should not find path with depth limit 1
        let result = traversal.depth_limited_search(id1, id3, 1, config).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_similarity_calculation() {
        let traversal = GraphTraversal::new();
        
        let concept1 = create_test_concept("Algebra", "math", vec!["equations", "variables"]);
        let concept2 = create_test_concept("Calculus", "math", vec!["derivatives", "integrals"]);
        let concept3 = create_test_concept("History", "social", vec!["dates", "events"]);
        
        let sim1_2 = traversal.calculate_concept_similarity(&concept1, &concept2);
        let sim1_3 = traversal.calculate_concept_similarity(&concept1, &concept3);
        
        // Concepts in same category should be more similar
        assert!(sim1_2 > sim1_3);
        assert!(sim1_2 >= 0.4); // At least category match
    }

    #[tokio::test]
    async fn test_filter_by_category() {
        let mut traversal = GraphTraversal::new();
        
        let concept1 = create_test_concept("Math", "math", vec![]);
        let concept2 = create_test_concept("Science", "science", vec![]);
        let concept3 = create_test_concept("Math2", "math", vec![]);
        
        let id1 = concept1.id;
        let id2 = concept2.id;
        let id3 = concept3.id;
        
        traversal.add_concepts(vec![concept1, concept2, concept3]);
        traversal.add_edges(vec![
            ConceptEdge { from: id1, to: id2, relationship: RelationshipType::Related, weight: 1.0 },
            ConceptEdge { from: id1, to: id3, relationship: RelationshipType::Related, weight: 1.0 },
        ]);

        let config = TraversalConfig {
            include_categories: Some(vec!["math".to_string()]),
            ..Default::default()
        };
        
        let result = traversal.bfs_traversal(id1, config).await.unwrap();
        
        // Should only include math concepts
        assert_eq!(result.visited_concepts.len(), 2);
        assert!(result.visited_concepts.contains(&id1));
        assert!(result.visited_concepts.contains(&id3));
        assert!(!result.visited_concepts.contains(&id2));
    }
}