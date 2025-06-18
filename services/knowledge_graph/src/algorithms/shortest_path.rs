//! Shortest path algorithms for knowledge graph traversal
//! 
//! Implements Dijkstra's algorithm and A* for finding optimal learning paths
//! between concepts, considering difficulty progression and prerequisites.

use crate::graph::Concept;
use anyhow::Result;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use tracing::{debug, info};
use uuid::Uuid;

/// Node in the path-finding algorithm with cost and heuristic
#[derive(Debug, Clone)]
pub struct PathNode {
    pub concept_id: Uuid,
    pub cost: f32,
    pub heuristic: f32,
    pub parent: Option<Uuid>,
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.concept_id == other.concept_id
    }
}

impl Eq for PathNode {}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap: reverse ordering for total cost (cost + heuristic)
        let self_total = self.cost + self.heuristic;
        let other_total = other.cost + other.heuristic;
        other_total.partial_cmp(&self_total).unwrap_or(Ordering::Equal)
    }
}

/// Edge in the knowledge graph with weight representing difficulty/cost
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub weight: f32,
    pub relationship: EdgeRelationship,
}

/// Types of relationships between concepts
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeRelationship {
    Prerequisite,
    Similarity,
    Progression,
    Related,
}

/// Result of a shortest path search
#[derive(Debug, Clone)]
pub struct PathResult {
    pub path: Vec<Uuid>,
    pub total_cost: f32,
    pub concepts: Vec<Concept>,
}

/// Shortest path algorithm implementation
#[derive(Clone)]
pub struct ShortestPath {
    edges: Vec<GraphEdge>,
    concepts: HashMap<Uuid, Concept>,
}

impl ShortestPath {
    /// Create a new shortest path finder
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            concepts: HashMap::new(),
        }
    }

    /// Add concepts to the graph
    pub fn add_concepts(&mut self, concepts: Vec<Concept>) {
        for concept in concepts {
            self.concepts.insert(concept.id, concept);
        }
    }

    /// Add edges to the graph
    pub fn add_edges(&mut self, edges: Vec<GraphEdge>) {
        self.edges.extend(edges);
    }

    /// Find shortest path using Dijkstra's algorithm
    pub async fn dijkstra_path(
        &self,
        start: Uuid,
        goal: Uuid,
        max_cost: Option<f32>,
    ) -> Result<Option<PathResult>> {
        info!("Finding shortest path from {:?} to {:?}", start, goal);
        
        if start == goal {
            return Ok(Some(PathResult {
                path: vec![start],
                total_cost: 0.0,
                concepts: vec![self.concepts.get(&start).unwrap().clone()],
            }));
        }

        let mut heap = BinaryHeap::new();
        let mut distances = HashMap::new();
        let mut parents = HashMap::new();
        let mut visited = HashSet::new();

        // Initialize starting node
        heap.push(PathNode {
            concept_id: start,
            cost: 0.0,
            heuristic: 0.0,
            parent: None,
        });
        distances.insert(start, 0.0);

        while let Some(current) = heap.pop() {
            if visited.contains(&current.concept_id) {
                continue;
            }

            visited.insert(current.concept_id);

            // Check if we reached the goal
            if current.concept_id == goal {
                let path = self.reconstruct_path(&parents, start, goal);
                let concepts = self.get_concepts_for_path(&path)?;
                
                debug!("Found path with cost: {}", current.cost);
                return Ok(Some(PathResult {
                    path,
                    total_cost: current.cost,
                    concepts,
                }));
            }

            // Check max cost constraint
            if let Some(max) = max_cost {
                if current.cost > max {
                    continue;
                }
            }

            // Explore neighbors
            for edge in self.get_outgoing_edges(current.concept_id) {
                if visited.contains(&edge.to) {
                    continue;
                }

                let new_cost = current.cost + edge.weight;
                let should_update = distances
                    .get(&edge.to)
                    .map_or(true, |&existing_cost| new_cost < existing_cost);

                if should_update {
                    distances.insert(edge.to, new_cost);
                    parents.insert(edge.to, current.concept_id);
                    
                    heap.push(PathNode {
                        concept_id: edge.to,
                        cost: new_cost,
                        heuristic: 0.0,
                        parent: Some(current.concept_id),
                    });
                }
            }
        }

        debug!("No path found from {:?} to {:?}", start, goal);
        Ok(None)
    }

    /// Find shortest path using A* algorithm with heuristic
    pub async fn astar_path(
        &self,
        start: Uuid,
        goal: Uuid,
        max_cost: Option<f32>,
    ) -> Result<Option<PathResult>> {
        info!("Finding A* path from {:?} to {:?}", start, goal);
        
        if start == goal {
            return Ok(Some(PathResult {
                path: vec![start],
                total_cost: 0.0,
                concepts: vec![self.concepts.get(&start).unwrap().clone()],
            }));
        }

        let mut heap = BinaryHeap::new();
        let mut g_scores = HashMap::new();
        let mut parents = HashMap::new();
        let mut visited = HashSet::new();

        // Get goal concept for heuristic calculation
        let goal_concept = self.concepts.get(&goal);

        // Initialize starting node
        let start_heuristic = goal_concept
            .and_then(|gc| self.concepts.get(&start).map(|sc| self.calculate_heuristic(sc, gc)))
            .unwrap_or(0.0);

        heap.push(PathNode {
            concept_id: start,
            cost: 0.0,
            heuristic: start_heuristic,
            parent: None,
        });
        g_scores.insert(start, 0.0);

        while let Some(current) = heap.pop() {
            if visited.contains(&current.concept_id) {
                continue;
            }

            visited.insert(current.concept_id);

            // Check if we reached the goal
            if current.concept_id == goal {
                let path = self.reconstruct_path(&parents, start, goal);
                let concepts = self.get_concepts_for_path(&path)?;
                
                debug!("Found A* path with cost: {}", current.cost);
                return Ok(Some(PathResult {
                    path,
                    total_cost: current.cost,
                    concepts,
                }));
            }

            // Check max cost constraint
            if let Some(max) = max_cost {
                if current.cost > max {
                    continue;
                }
            }

            // Explore neighbors
            for edge in self.get_outgoing_edges(current.concept_id) {
                if visited.contains(&edge.to) {
                    continue;
                }

                let tentative_g_score = current.cost + edge.weight;
                let should_update = g_scores
                    .get(&edge.to)
                    .map_or(true, |&existing_cost| tentative_g_score < existing_cost);

                if should_update {
                    let heuristic = goal_concept
                        .and_then(|gc| self.concepts.get(&edge.to).map(|tc| self.calculate_heuristic(tc, gc)))
                        .unwrap_or(0.0);

                    g_scores.insert(edge.to, tentative_g_score);
                    parents.insert(edge.to, current.concept_id);
                    
                    heap.push(PathNode {
                        concept_id: edge.to,
                        cost: tentative_g_score,
                        heuristic,
                        parent: Some(current.concept_id),
                    });
                }
            }
        }

        debug!("No A* path found from {:?} to {:?}", start, goal);
        Ok(None)
    }

    /// Find multiple alternative paths
    pub async fn find_alternative_paths(
        &self,
        start: Uuid,
        goal: Uuid,
        num_paths: usize,
        max_cost: Option<f32>,
    ) -> Result<Vec<PathResult>> {
        info!("Finding {} alternative paths from {:?} to {:?}", num_paths, start, goal);
        
        let mut paths = Vec::new();
        let mut blocked_edges = HashSet::new();

        for i in 0..num_paths {
            // Find path with current blocked edges
            if let Some(path) = self.dijkstra_path_with_blocked(start, goal, max_cost, &blocked_edges).await? {
                // Block the most expensive edge in this path for next iteration
                if let Some(edge_to_block) = self.find_most_expensive_edge_in_path(&path.path) {
                    blocked_edges.insert((edge_to_block.from, edge_to_block.to));
                }
                
                paths.push(path);
                debug!("Found alternative path {}: {} nodes", i + 1, paths.last().unwrap().path.len());
            } else {
                break;
            }
        }

        Ok(paths)
    }

    /// Calculate heuristic for A* (estimated cost to goal)
    fn calculate_heuristic(&self, current: &Concept, goal: &Concept) -> f32 {
        // Use difficulty difference and category similarity as heuristic
        let difficulty_diff = self.difficulty_to_float(&goal.difficulty) - self.difficulty_to_float(&current.difficulty);
        let category_bonus = if current.category == goal.category { -1.0 } else { 0.0 };
        
        // Ensure heuristic is admissible (never overestimates)
        (difficulty_diff + category_bonus).max(0.0)
    }

    /// Convert difficulty string to numeric value
    fn difficulty_to_float(&self, difficulty: &str) -> f32 {
        match difficulty.to_lowercase().as_str() {
            "beginner" => 1.0,
            "intermediate" => 2.0,
            "advanced" => 3.0,
            "expert" => 4.0,
            _ => 2.0, // default to intermediate
        }
    }

    /// Get outgoing edges from a concept
    fn get_outgoing_edges(&self, concept_id: Uuid) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|edge| edge.from == concept_id).collect()
    }

    /// Reconstruct path from parent map
    fn reconstruct_path(&self, parents: &HashMap<Uuid, Uuid>, start: Uuid, goal: Uuid) -> Vec<Uuid> {
        let mut path = Vec::new();
        let mut current = goal;
        
        path.push(current);
        while let Some(&parent) = parents.get(&current) {
            path.push(parent);
            current = parent;
            if current == start {
                break;
            }
        }
        
        path.reverse();
        path
    }

    /// Get concepts for a path
    fn get_concepts_for_path(&self, path: &[Uuid]) -> Result<Vec<Concept>> {
        path.iter()
            .map(|&id| {
                self.concepts.get(&id)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Concept not found: {:?}", id))
            })
            .collect()
    }

    /// Dijkstra with blocked edges (for alternative path finding)
    async fn dijkstra_path_with_blocked(
        &self,
        start: Uuid,
        goal: Uuid,
        max_cost: Option<f32>,
        blocked_edges: &HashSet<(Uuid, Uuid)>,
    ) -> Result<Option<PathResult>> {
        let mut heap = BinaryHeap::new();
        let mut distances = HashMap::new();
        let mut parents = HashMap::new();
        let mut visited = HashSet::new();

        heap.push(PathNode {
            concept_id: start,
            cost: 0.0,
            heuristic: 0.0,
            parent: None,
        });
        distances.insert(start, 0.0);

        while let Some(current) = heap.pop() {
            if visited.contains(&current.concept_id) {
                continue;
            }

            visited.insert(current.concept_id);

            if current.concept_id == goal {
                let path = self.reconstruct_path(&parents, start, goal);
                let concepts = self.get_concepts_for_path(&path)?;
                
                return Ok(Some(PathResult {
                    path,
                    total_cost: current.cost,
                    concepts,
                }));
            }

            if let Some(max) = max_cost {
                if current.cost > max {
                    continue;
                }
            }

            for edge in self.get_outgoing_edges(current.concept_id) {
                // Skip blocked edges
                if blocked_edges.contains(&(edge.from, edge.to)) {
                    continue;
                }
                
                if visited.contains(&edge.to) {
                    continue;
                }

                let new_cost = current.cost + edge.weight;
                let should_update = distances
                    .get(&edge.to)
                    .map_or(true, |&existing_cost| new_cost < existing_cost);

                if should_update {
                    distances.insert(edge.to, new_cost);
                    parents.insert(edge.to, current.concept_id);
                    
                    heap.push(PathNode {
                        concept_id: edge.to,
                        cost: new_cost,
                        heuristic: 0.0,
                        parent: Some(current.concept_id),
                    });
                }
            }
        }

        Ok(None)
    }

    /// Find the most expensive edge in a path
    fn find_most_expensive_edge_in_path(&self, path: &[Uuid]) -> Option<&GraphEdge> {
        let mut most_expensive = None;
        let mut max_weight = 0.0;

        for window in path.windows(2) {
            if let [from, to] = window {
                for edge in &self.edges {
                    if edge.from == *from && edge.to == *to && edge.weight > max_weight {
                        max_weight = edge.weight;
                        most_expensive = Some(edge);
                    }
                }
            }
        }

        most_expensive
    }
}

impl Default for ShortestPath {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_concept(name: &str, difficulty: &str, category: &str) -> Concept {
        Concept {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: Some(format!("Test concept: {}", name)),
            difficulty: difficulty.to_string(),
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
    async fn test_dijkstra_simple_path() {
        let mut shortest_path = ShortestPath::new();
        
        let concept1 = create_test_concept("Concept 1", "beginner", "math");
        let concept2 = create_test_concept("Concept 2", "intermediate", "math");
        let concept3 = create_test_concept("Concept 3", "advanced", "math");
        
        let id1 = concept1.id;
        let id2 = concept2.id;
        let id3 = concept3.id;
        
        shortest_path.add_concepts(vec![concept1, concept2, concept3]);
        shortest_path.add_edges(vec![
            GraphEdge {
                from: id1,
                to: id2,
                weight: 1.0,
                relationship: EdgeRelationship::Prerequisite,
            },
            GraphEdge {
                from: id2,
                to: id3,
                weight: 2.0,
                relationship: EdgeRelationship::Progression,
            },
        ]);

        let result = shortest_path.dijkstra_path(id1, id3, None).await.unwrap();
        
        assert!(result.is_some());
        let path_result = result.unwrap();
        assert_eq!(path_result.path, vec![id1, id2, id3]);
        assert_eq!(path_result.total_cost, 3.0);
    }

    #[tokio::test]
    async fn test_astar_with_heuristic() {
        let mut shortest_path = ShortestPath::new();
        
        let concept1 = create_test_concept("Start", "beginner", "math");
        let concept2 = create_test_concept("Goal", "advanced", "math");
        
        let id1 = concept1.id;
        let id2 = concept2.id;
        
        shortest_path.add_concepts(vec![concept1, concept2]);
        shortest_path.add_edges(vec![
            GraphEdge {
                from: id1,
                to: id2,
                weight: 1.0,
                relationship: EdgeRelationship::Progression,
            },
        ]);

        let result = shortest_path.astar_path(id1, id2, None).await.unwrap();
        
        assert!(result.is_some());
        let path_result = result.unwrap();
        assert_eq!(path_result.path, vec![id1, id2]);
        assert_eq!(path_result.total_cost, 1.0);
    }

    #[tokio::test]
    async fn test_no_path_exists() {
        let shortest_path = ShortestPath::new();
        
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let result = shortest_path.dijkstra_path(id1, id2, None).await.unwrap();
        
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_alternative_paths() {
        let mut shortest_path = ShortestPath::new();
        
        let concept1 = create_test_concept("Start", "beginner", "math");
        let concept2 = create_test_concept("Middle1", "intermediate", "math");
        let concept3 = create_test_concept("Middle2", "intermediate", "math");
        let concept4 = create_test_concept("Goal", "advanced", "math");
        
        let id1 = concept1.id;
        let id2 = concept2.id;
        let id3 = concept3.id;
        let id4 = concept4.id;
        
        shortest_path.add_concepts(vec![concept1, concept2, concept3, concept4]);
        shortest_path.add_edges(vec![
            GraphEdge { from: id1, to: id2, weight: 1.0, relationship: EdgeRelationship::Prerequisite },
            GraphEdge { from: id1, to: id3, weight: 2.0, relationship: EdgeRelationship::Prerequisite },
            GraphEdge { from: id2, to: id4, weight: 1.0, relationship: EdgeRelationship::Progression },
            GraphEdge { from: id3, to: id4, weight: 1.0, relationship: EdgeRelationship::Progression },
        ]);

        let paths = shortest_path.find_alternative_paths(id1, id4, 2, None).await.unwrap();
        
        assert_eq!(paths.len(), 2);
        assert!(paths[0].total_cost <= paths[1].total_cost);
    }
}