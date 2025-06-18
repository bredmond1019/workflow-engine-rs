//! Ranking algorithms for concept importance and learning paths
//! 
//! Implements PageRank, community detection, and centrality measures
//! to identify important concepts and optimal learning sequences.

use crate::graph::Concept;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};
use uuid::Uuid;

/// Result of a ranking algorithm
#[derive(Debug, Clone)]
pub struct RankingResult {
    pub rankings: HashMap<Uuid, f32>,
    pub concepts: Vec<Concept>,
    pub metadata: RankingMetadata,
}

/// Metadata about the ranking process
#[derive(Debug, Clone)]
pub struct RankingMetadata {
    pub algorithm: String,
    pub iterations: usize,
    pub convergence_error: f32,
    pub total_concepts: usize,
}

/// Community detection result
#[derive(Debug, Clone)]
pub struct CommunityResult {
    pub communities: Vec<Vec<Uuid>>,
    pub modularity: f32,
    pub concept_to_community: HashMap<Uuid, usize>,
}

/// Edge for ranking algorithms
#[derive(Debug, Clone)]
pub struct RankingEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub weight: f32,
}

/// Configuration for ranking algorithms
#[derive(Debug, Clone)]
pub struct RankingConfig {
    pub damping_factor: f32,
    pub tolerance: f32,
    pub max_iterations: usize,
    pub normalize_weights: bool,
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            damping_factor: 0.85,
            tolerance: 1e-6,
            max_iterations: 100,
            normalize_weights: true,
        }
    }
}

/// Ranking algorithms implementation
#[derive(Clone)]
pub struct ConceptRanking {
    concepts: HashMap<Uuid, Concept>,
    edges: Vec<RankingEdge>,
}

impl ConceptRanking {
    /// Create a new concept ranking instance
    pub fn new() -> Self {
        Self {
            concepts: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Add concepts to the ranking system
    pub fn add_concepts(&mut self, concepts: Vec<Concept>) {
        for concept in concepts {
            self.concepts.insert(concept.id, concept);
        }
    }

    /// Add edges between concepts
    pub fn add_edges(&mut self, edges: Vec<RankingEdge>) {
        self.edges.extend(edges);
    }

    /// Calculate PageRank scores for concepts
    pub async fn pagerank(&self, config: RankingConfig) -> Result<RankingResult> {
        info!("Starting PageRank calculation for {} concepts", self.concepts.len());
        
        let concept_ids: Vec<Uuid> = self.concepts.keys().cloned().collect();
        let n = concept_ids.len();
        
        if n == 0 {
            return Ok(RankingResult {
                rankings: HashMap::new(),
                concepts: Vec::new(),
                metadata: RankingMetadata {
                    algorithm: "PageRank".to_string(),
                    iterations: 0,
                    convergence_error: 0.0,
                    total_concepts: 0,
                },
            });
        }

        // Build adjacency matrix
        let mut out_links: HashMap<Uuid, Vec<(Uuid, f32)>> = HashMap::new();
        let mut in_links: HashMap<Uuid, Vec<(Uuid, f32)>> = HashMap::new();
        
        for concept_id in &concept_ids {
            out_links.insert(*concept_id, Vec::new());
            in_links.insert(*concept_id, Vec::new());
        }

        for edge in &self.edges {
            if concept_ids.contains(&edge.from) && concept_ids.contains(&edge.to) {
                let weight = if config.normalize_weights { 
                    edge.weight 
                } else { 
                    1.0 
                };
                
                out_links.get_mut(&edge.from).unwrap().push((edge.to, weight));
                in_links.get_mut(&edge.to).unwrap().push((edge.from, weight));
            }
        }

        // Normalize outgoing weights
        for (_, out_edges) in &mut out_links {
            if !out_edges.is_empty() {
                let total_weight: f32 = out_edges.iter().map(|(_, w)| w).sum();
                if total_weight > 0.0 {
                    for (_, weight) in out_edges {
                        *weight /= total_weight;
                    }
                }
            }
        }

        // Initialize PageRank values
        let initial_value = 1.0 / n as f32;
        let mut current_ranks: HashMap<Uuid, f32> = concept_ids.iter()
            .map(|&id| (id, initial_value))
            .collect();
        let mut new_ranks = current_ranks.clone();

        let mut iterations = 0;
        let mut convergence_error = f32::INFINITY;

        // Iterate until convergence
        while iterations < config.max_iterations && convergence_error > config.tolerance {
            for concept_id in &concept_ids {
                let mut rank_sum = 0.0;
                
                // Sum contributions from incoming links
                if let Some(incoming) = in_links.get(concept_id) {
                    for &(from_id, weight) in incoming {
                        if let Some(&from_rank) = current_ranks.get(&from_id) {
                            rank_sum += from_rank * weight;
                        }
                    }
                }

                // Apply PageRank formula: (1-d)/N + d * sum
                new_ranks.insert(
                    *concept_id,
                    (1.0 - config.damping_factor) / n as f32 + config.damping_factor * rank_sum,
                );
            }

            // Calculate convergence error
            convergence_error = concept_ids.iter()
                .map(|&id| {
                    let old = current_ranks.get(&id).unwrap_or(&0.0);
                    let new = new_ranks.get(&id).unwrap_or(&0.0);
                    (old - new).abs()
                })
                .sum::<f32>();

            current_ranks = new_ranks.clone();
            iterations += 1;

            if iterations % 10 == 0 {
                debug!("PageRank iteration {}, convergence error: {}", iterations, convergence_error);
            }
        }

        let concepts = self.get_concepts_for_ids(&concept_ids)?;

        info!("PageRank completed in {} iterations, final error: {}", iterations, convergence_error);

        Ok(RankingResult {
            rankings: current_ranks,
            concepts,
            metadata: RankingMetadata {
                algorithm: "PageRank".to_string(),
                iterations,
                convergence_error,
                total_concepts: n,
            },
        })
    }

    /// Calculate centrality measures
    pub async fn centrality_measures(&self) -> Result<HashMap<String, HashMap<Uuid, f32>>> {
        info!("Calculating centrality measures");
        
        let concept_ids: Vec<Uuid> = self.concepts.keys().cloned().collect();
        let mut measures = HashMap::new();

        // Degree centrality
        let degree_centrality = self.calculate_degree_centrality(&concept_ids);
        measures.insert("degree".to_string(), degree_centrality);

        // Betweenness centrality (simplified version)
        let betweenness_centrality = self.calculate_betweenness_centrality(&concept_ids).await?;
        measures.insert("betweenness".to_string(), betweenness_centrality);

        // Closeness centrality
        let closeness_centrality = self.calculate_closeness_centrality(&concept_ids).await?;
        measures.insert("closeness".to_string(), closeness_centrality);

        debug!("Calculated {} centrality measures for {} concepts", 
               measures.len(), concept_ids.len());

        Ok(measures)
    }

    /// Detect communities using Louvain algorithm (simplified)
    pub async fn detect_communities(&self) -> Result<CommunityResult> {
        info!("Detecting communities in concept graph");
        
        let concept_ids: Vec<Uuid> = self.concepts.keys().cloned().collect();
        let n = concept_ids.len();

        if n == 0 {
            return Ok(CommunityResult {
                communities: Vec::new(),
                modularity: 0.0,
                concept_to_community: HashMap::new(),
            });
        }

        // Build adjacency matrix
        let mut adjacency: HashMap<(Uuid, Uuid), f32> = HashMap::new();
        let mut node_weights: HashMap<Uuid, f32> = HashMap::new();
        let mut total_weight = 0.0;

        for concept_id in &concept_ids {
            node_weights.insert(*concept_id, 0.0);
        }

        for edge in &self.edges {
            if concept_ids.contains(&edge.from) && concept_ids.contains(&edge.to) {
                adjacency.insert((edge.from, edge.to), edge.weight);
                *node_weights.get_mut(&edge.from).unwrap() += edge.weight;
                *node_weights.get_mut(&edge.to).unwrap() += edge.weight;
                total_weight += edge.weight;
            }
        }

        // Initialize: each node in its own community
        let mut communities: HashMap<Uuid, usize> = concept_ids.iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();

        // Greedy optimization (simplified Louvain)
        let mut improved = true;
        let mut iteration = 0;
        
        while improved && iteration < 10 {
            improved = false;
            iteration += 1;

            for &node in &concept_ids {
                let current_community = communities[&node];
                let mut best_community = current_community;
                let mut best_gain = 0.0;

                // Try moving node to neighbor communities
                let neighbors = self.get_neighbors(node);
                let mut neighbor_communities: HashSet<usize> = neighbors.iter()
                    .map(|&neighbor| communities[&neighbor])
                    .collect();
                neighbor_communities.insert(current_community);

                for &community in &neighbor_communities {
                    if community != current_community {
                        let gain = self.calculate_modularity_gain(
                            node,
                            current_community,
                            community,
                            &communities,
                            &adjacency,
                            &node_weights,
                            total_weight,
                        );

                        if gain > best_gain {
                            best_gain = gain;
                            best_community = community;
                        }
                    }
                }

                if best_community != current_community {
                    communities.insert(node, best_community);
                    improved = true;
                }
            }
        }

        // Build final community structure
        let mut community_map: HashMap<usize, Vec<Uuid>> = HashMap::new();
        for (&node, &community) in &communities {
            community_map.entry(community).or_default().push(node);
        }

        let final_communities: Vec<Vec<Uuid>> = community_map.into_values().collect();
        let modularity = self.calculate_modularity(&communities, &adjacency, &node_weights, total_weight);

        debug!("Community detection completed: {} communities, modularity: {}", 
               final_communities.len(), modularity);

        Ok(CommunityResult {
            communities: final_communities,
            modularity,
            concept_to_community: communities,
        })
    }

    /// Rank concepts by learning difficulty progression
    pub async fn difficulty_ranking(&self) -> Result<RankingResult> {
        info!("Ranking concepts by difficulty progression");
        
        let concept_ids: Vec<Uuid> = self.concepts.keys().cloned().collect();
        let mut rankings = HashMap::new();

        for &concept_id in &concept_ids {
            if let Some(concept) = self.concepts.get(&concept_id) {
                let difficulty_score = match concept.difficulty.to_lowercase().as_str() {
                    "beginner" => 1.0,
                    "intermediate" => 2.0,
                    "advanced" => 3.0,
                    "expert" => 4.0,
                    _ => 2.0,
                };

                // Combine with quality score
                let final_score = difficulty_score * concept.quality_score;
                rankings.insert(concept_id, final_score);
            }
        }

        let concepts = self.get_concepts_for_ids(&concept_ids)?;

        Ok(RankingResult {
            rankings,
            concepts,
            metadata: RankingMetadata {
                algorithm: "Difficulty Ranking".to_string(),
                iterations: 1,
                convergence_error: 0.0,
                total_concepts: concept_ids.len(),
            },
        })
    }

    /// Calculate degree centrality
    fn calculate_degree_centrality(&self, concept_ids: &[Uuid]) -> HashMap<Uuid, f32> {
        let mut degree_counts: HashMap<Uuid, usize> = concept_ids.iter()
            .map(|&id| (id, 0))
            .collect();

        for edge in &self.edges {
            if concept_ids.contains(&edge.from) {
                *degree_counts.get_mut(&edge.from).unwrap() += 1;
            }
            if concept_ids.contains(&edge.to) {
                *degree_counts.get_mut(&edge.to).unwrap() += 1;
            }
        }

        let max_degree = degree_counts.values().max().cloned().unwrap_or(1) as f32;
        
        degree_counts.into_iter()
            .map(|(id, count)| (id, count as f32 / max_degree))
            .collect()
    }

    /// Calculate betweenness centrality (simplified)
    async fn calculate_betweenness_centrality(&self, concept_ids: &[Uuid]) -> Result<HashMap<Uuid, f32>> {
        let mut betweenness: HashMap<Uuid, f32> = concept_ids.iter()
            .map(|&id| (id, 0.0))
            .collect();

        // For each pair of nodes, find shortest paths
        for &source in concept_ids {
            for &target in concept_ids {
                if source != target {
                    if let Some(paths) = self.find_all_shortest_paths(source, target).await? {
                        let path_count = paths.len() as f32;
                        
                        for path in paths {
                            // Count intermediate nodes
                            for &intermediate in &path[1..path.len()-1] {
                                *betweenness.get_mut(&intermediate).unwrap() += 1.0 / path_count;
                            }
                        }
                    }
                }
            }
        }

        // Normalize
        let n = concept_ids.len() as f32;
        let normalization_factor = if n > 2.0 { (n - 1.0) * (n - 2.0) / 2.0 } else { 1.0 };
        
        for value in betweenness.values_mut() {
            *value /= normalization_factor;
        }

        Ok(betweenness)
    }

    /// Calculate closeness centrality
    async fn calculate_closeness_centrality(&self, concept_ids: &[Uuid]) -> Result<HashMap<Uuid, f32>> {
        let mut closeness: HashMap<Uuid, f32> = HashMap::new();

        for &concept_id in concept_ids {
            let mut total_distance = 0.0;
            let mut reachable_count = 0;

            for &other_id in concept_ids {
                if concept_id != other_id {
                    if let Some(distance) = self.shortest_path_distance(concept_id, other_id).await? {
                        total_distance += distance;
                        reachable_count += 1;
                    }
                }
            }

            let centrality = if reachable_count > 0 && total_distance > 0.0 {
                (reachable_count as f32) / total_distance
            } else {
                0.0
            };

            closeness.insert(concept_id, centrality);
        }

        Ok(closeness)
    }

    /// Get neighbors of a concept
    fn get_neighbors(&self, concept_id: Uuid) -> Vec<Uuid> {
        let mut neighbors = Vec::new();
        
        for edge in &self.edges {
            if edge.from == concept_id {
                neighbors.push(edge.to);
            } else if edge.to == concept_id {
                neighbors.push(edge.from);
            }
        }
        
        neighbors
    }

    /// Calculate modularity gain for moving a node to a different community
    fn calculate_modularity_gain(
        &self,
        node: Uuid,
        from_community: usize,
        to_community: usize,
        communities: &HashMap<Uuid, usize>,
        adjacency: &HashMap<(Uuid, Uuid), f32>,
        node_weights: &HashMap<Uuid, f32>,
        total_weight: f32,
    ) -> f32 {
        // Simplified modularity gain calculation
        let _node_weight = node_weights.get(&node).cloned().unwrap_or(0.0);
        let neighbors = self.get_neighbors(node);
        
        let mut internal_edges_from = 0.0;
        let mut internal_edges_to = 0.0;
        
        for neighbor in neighbors {
            let neighbor_community = communities.get(&neighbor).cloned().unwrap_or(0);
            let edge_weight = adjacency.get(&(node, neighbor))
                .or_else(|| adjacency.get(&(neighbor, node)))
                .cloned()
                .unwrap_or(0.0);
            
            if neighbor_community == from_community {
                internal_edges_from += edge_weight;
            }
            if neighbor_community == to_community {
                internal_edges_to += edge_weight;
            }
        }
        
        // Simplified gain calculation
        (internal_edges_to - internal_edges_from) / total_weight
    }

    /// Calculate overall modularity
    fn calculate_modularity(
        &self,
        communities: &HashMap<Uuid, usize>,
        adjacency: &HashMap<(Uuid, Uuid), f32>,
        node_weights: &HashMap<Uuid, f32>,
        total_weight: f32,
    ) -> f32 {
        let mut modularity = 0.0;
        
        for ((u, v), &weight) in adjacency {
            let community_u = communities.get(u).cloned().unwrap_or(0);
            let community_v = communities.get(v).cloned().unwrap_or(0);
            
            if community_u == community_v {
                let degree_u = node_weights.get(u).cloned().unwrap_or(0.0);
                let degree_v = node_weights.get(v).cloned().unwrap_or(0.0);
                let expected = (degree_u * degree_v) / (2.0 * total_weight);
                modularity += weight - expected;
            }
        }
        
        modularity / (2.0 * total_weight)
    }

    /// Find all shortest paths between two concepts (simplified BFS)
    async fn find_all_shortest_paths(&self, source: Uuid, target: Uuid) -> Result<Option<Vec<Vec<Uuid>>>> {
        // Simplified: just return one shortest path for now
        if let Some(_distance) = self.shortest_path_distance(source, target).await? {
            Ok(Some(vec![vec![source, target]]))
        } else {
            Ok(None)
        }
    }

    /// Calculate shortest path distance between two concepts
    async fn shortest_path_distance(&self, source: Uuid, target: Uuid) -> Result<Option<f32>> {
        use std::collections::VecDeque;
        
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut distances = HashMap::new();
        
        queue.push_back(source);
        visited.insert(source);
        distances.insert(source, 0.0);
        
        while let Some(current) = queue.pop_front() {
            if current == target {
                return Ok(distances.get(&target).cloned());
            }
            
            let current_distance = distances.get(&current).cloned().unwrap_or(0.0);
            let neighbors = self.get_neighbors(current);
            
            for neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    distances.insert(neighbor, current_distance + 1.0);
                    queue.push_back(neighbor);
                }
            }
        }
        
        Ok(None)
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

impl Default for ConceptRanking {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_concept(name: &str, difficulty: &str, quality: f32) -> Concept {
        Concept {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: Some(format!("Test concept: {}", name)),
            difficulty: difficulty.to_string(),
            category: "math".to_string(),
            subcategory: None,
            tags: vec![],
            quality_score: quality,
            estimated_time: Some(60.0),
            embeddings: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
        }
    }

    #[tokio::test]
    async fn test_pagerank_simple() {
        let mut ranking = ConceptRanking::new();
        
        let concept1 = create_test_concept("Concept 1", "beginner", 0.8);
        let concept2 = create_test_concept("Concept 2", "intermediate", 0.9);
        let concept3 = create_test_concept("Concept 3", "advanced", 0.7);
        
        let id1 = concept1.id;
        let id2 = concept2.id;
        let id3 = concept3.id;
        
        ranking.add_concepts(vec![concept1, concept2, concept3]);
        ranking.add_edges(vec![
            RankingEdge { from: id1, to: id2, weight: 1.0 },
            RankingEdge { from: id2, to: id3, weight: 1.0 },
            RankingEdge { from: id3, to: id1, weight: 1.0 },
        ]);

        let config = RankingConfig::default();
        let result = ranking.pagerank(config).await.unwrap();
        
        // All concepts should have similar PageRank in a simple cycle
        assert_eq!(result.rankings.len(), 3);
        
        let rank1 = result.rankings[&id1];
        let rank2 = result.rankings[&id2];
        let rank3 = result.rankings[&id3];
        
        // In a symmetric graph, ranks should be similar
        assert!((rank1 - rank2).abs() < 0.1);
        assert!((rank2 - rank3).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_difficulty_ranking() {
        let mut ranking = ConceptRanking::new();
        
        let concept1 = create_test_concept("Beginner", "beginner", 1.0);
        let concept2 = create_test_concept("Expert", "expert", 1.0);
        
        let id1 = concept1.id;
        let id2 = concept2.id;
        
        ranking.add_concepts(vec![concept1, concept2]);

        let result = ranking.difficulty_ranking().await.unwrap();
        
        // Expert concept should have higher difficulty score
        assert!(result.rankings[&id2] > result.rankings[&id1]);
    }

    #[tokio::test]
    async fn test_degree_centrality() {
        let mut ranking = ConceptRanking::new();
        
        let concept1 = create_test_concept("Hub", "intermediate", 0.8);
        let concept2 = create_test_concept("Leaf 1", "intermediate", 0.8);
        let concept3 = create_test_concept("Leaf 2", "intermediate", 0.8);
        
        let id1 = concept1.id;
        let id2 = concept2.id;
        let id3 = concept3.id;
        
        ranking.add_concepts(vec![concept1, concept2, concept3]);
        ranking.add_edges(vec![
            RankingEdge { from: id1, to: id2, weight: 1.0 },
            RankingEdge { from: id1, to: id3, weight: 1.0 },
        ]);

        let concept_ids = vec![id1, id2, id3];
        let centrality = ranking.calculate_degree_centrality(&concept_ids);
        
        // Hub should have highest degree centrality
        assert!(centrality[&id1] > centrality[&id2]);
        assert!(centrality[&id1] > centrality[&id3]);
    }

    #[tokio::test]
    async fn test_community_detection() {
        let mut ranking = ConceptRanking::new();
        
        // Create two clusters
        let concepts: Vec<Concept> = (0..6).map(|i| {
            create_test_concept(&format!("Concept {}", i), "intermediate", 0.8)
        }).collect();
        
        let ids: Vec<Uuid> = concepts.iter().map(|c| c.id).collect();
        ranking.add_concepts(concepts);
        
        // Create edges within clusters
        ranking.add_edges(vec![
            // Cluster 1: 0-1-2
            RankingEdge { from: ids[0], to: ids[1], weight: 1.0 },
            RankingEdge { from: ids[1], to: ids[2], weight: 1.0 },
            RankingEdge { from: ids[0], to: ids[2], weight: 1.0 },
            // Cluster 2: 3-4-5
            RankingEdge { from: ids[3], to: ids[4], weight: 1.0 },
            RankingEdge { from: ids[4], to: ids[5], weight: 1.0 },
            RankingEdge { from: ids[3], to: ids[5], weight: 1.0 },
            // Weak connection between clusters
            RankingEdge { from: ids[1], to: ids[4], weight: 0.1 },
        ]);

        let result = ranking.detect_communities().await.unwrap();
        
        // Should detect communities (exact number may vary due to algorithm)
        assert!(result.communities.len() >= 2);
        assert!(result.modularity > 0.0);
    }
}