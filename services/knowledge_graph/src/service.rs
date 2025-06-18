//! High-level knowledge graph service
//! 
//! Integrates Dgraph queries with graph algorithms to provide
//! comprehensive knowledge graph operations and relationship discovery.

use crate::algorithms::{GraphAlgorithms, shortest_path::{GraphEdge, EdgeRelationship}, traversal::{TraversalConfig, ConceptEdge, RelationshipType}};
use crate::client::{DgraphClient, DgraphResponseParser};
use crate::graph::{Concept, DgraphConfig, GraphDatabase};
use crate::query::{QueryBuilder, QueryType, QueryParameters, QueryConstraints};
use crate::error::{KnowledgeGraphError, Result as KgResult, ErrorContext};
use anyhow::{anyhow, Context, Result};
use anyhow::Context as AnyhowContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipDiscoveryRequest {
    pub concept_id: String,
    pub max_depth: Option<u32>,
    pub relationship_types: Option<Vec<String>>,
    pub min_strength: Option<f32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathFindingRequest {
    pub from_concept: String,
    pub to_concept: String,
    pub algorithm: Option<String>, // "dijkstra", "astar", "bfs"
    pub max_cost: Option<f32>,
    pub constraints: Option<QueryConstraints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipResult {
    pub concept: Concept,
    pub relationships: Vec<ConceptRelationship>,
    pub total_found: usize,
    pub max_depth_reached: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptRelationship {
    pub target_concept: Concept,
    pub relationship_type: String,
    pub strength: f32,
    pub path_length: u32,
    pub shared_properties: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathFindingResult {
    pub path: Vec<Concept>,
    pub total_cost: f32,
    pub algorithm_used: String,
    pub metadata: PathMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathMetadata {
    pub alternative_paths: Option<Vec<Vec<Uuid>>>,
    pub bottlenecks: Vec<Uuid>,
    pub difficulty_progression: Vec<String>,
    pub estimated_time: f32,
}

/// High-level knowledge graph service
pub struct KnowledgeGraphService {
    client: Arc<DgraphClient>,
    graph_db: Arc<GraphDatabase>,
    algorithms: GraphAlgorithms,
    query_builder: QueryBuilder,
    response_parser: DgraphResponseParser,
}

impl KnowledgeGraphService {
    /// Create a new knowledge graph service
    pub async fn new(config: DgraphConfig) -> Result<Self> {
        info!("Initializing Knowledge Graph Service");
        
        // Create clients
        let endpoint = format!("{}:{}", config.host, config.grpc_port);
        let client = Arc::new(DgraphClient::new(endpoint).await
            .map_err(|e| anyhow::Error::from(e))
            .context("Failed to create Dgraph client")?);
        
        let graph_db = Arc::new(GraphDatabase::new(config).await
            .map_err(|e| anyhow::Error::from(e))
            .context("Failed to create graph database")?);
        
        let algorithms = GraphAlgorithms::new();
        let query_builder = QueryBuilder::new();
        let response_parser = DgraphResponseParser::new();
        
        info!("Knowledge Graph Service initialized successfully");
        
        Ok(Self {
            client,
            graph_db,
            algorithms,
            query_builder,
            response_parser,
        })
    }

    /// Discover relationships for a given concept
    pub async fn discover_relationships(&self, request: RelationshipDiscoveryRequest) -> Result<RelationshipResult> {
        info!("Discovering relationships for concept: {}", request.concept_id);
        
        // First, get the base concept
        let base_concept = self.get_concept_by_id(&request.concept_id).await?;
        
        // Build traversal configuration
        let config = TraversalConfig {
            max_depth: request.max_depth.map(|d| d as usize),
            max_nodes: request.limit.map(|l| l as usize),
            min_quality_score: request.min_strength,
            follow_relationships: self.parse_relationship_types(request.relationship_types),
            ..Default::default()
        };
        
        // Load concept data into algorithms
        let concepts = self.load_related_concepts(&base_concept.id, config.max_depth.unwrap_or(3)).await?;
        let edges = self.build_concept_edges(&concepts).await?;
        
        let mut algorithms = self.algorithms.clone();
        algorithms.add_concepts(concepts.clone());
        algorithms.traversal_engine_mut().add_edges(edges);
        
        // Perform BFS traversal to find related concepts
        let traversal_result = algorithms.bfs_traversal(base_concept.id, config).await?;
        
        // Calculate relationship strengths and build results
        let mut relationships = Vec::new();
        let max_depth_reached = traversal_result.distances.values().max().copied().unwrap_or(0) as u32;
        
        for visited_id in &traversal_result.visited_concepts {
            if *visited_id != base_concept.id {
                if let Some(target_concept) = concepts.iter().find(|c| c.id == *visited_id) {
                    let relationship = self.analyze_concept_relationship(&base_concept, target_concept, &traversal_result).await?;
                    relationships.push(relationship);
                }
            }
        }
        
        // Sort by strength (descending)
        relationships.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap_or(std::cmp::Ordering::Equal));
        
        // Apply limit
        if let Some(limit) = request.limit {
            relationships.truncate(limit as usize);
        }
        
        info!("Found {} relationships for concept {}", relationships.len(), request.concept_id);
        
        Ok(RelationshipResult {
            concept: base_concept,
            relationships,
            total_found: traversal_result.visited_concepts.len() - 1, // Exclude base concept
            max_depth_reached,
        })
    }

    /// Find optimal path between two concepts
    pub async fn find_path(&self, request: PathFindingRequest) -> Result<PathFindingResult> {
        info!("Finding path from {} to {}", request.from_concept, request.to_concept);
        
        // Get concepts
        let from_concept = self.get_concept_by_name(&request.from_concept).await?;
        let to_concept = self.get_concept_by_name(&request.to_concept).await?;
        
        // Load graph data
        let concepts = self.load_path_concepts(&from_concept.id, &to_concept.id, 10).await?;
        let edges = self.build_graph_edges(&concepts).await?;
        
        let mut algorithms = self.algorithms.clone();
        algorithms.add_concepts(concepts.clone());
        algorithms.shortest_path_engine_mut().add_edges(edges);
        
        // Choose algorithm
        let algorithm = request.algorithm.as_deref().unwrap_or("dijkstra");
        let path_result = match algorithm {
            "dijkstra" => {
                algorithms.shortest_path(from_concept.id, to_concept.id, request.max_cost).await?
            }
            "astar" => {
                algorithms.shortest_path_engine().astar_path(from_concept.id, to_concept.id, request.max_cost).await?
            }
            "bfs" => {
                let config = TraversalConfig {
                    max_depth: Some(10),
                    ..Default::default()
                };
                let traversal = algorithms.bfs_traversal(from_concept.id, config).await?;
                self.extract_path_from_traversal(traversal, from_concept.id, to_concept.id)?
            }
            _ => return Err(anyhow!("Unknown algorithm: {}", algorithm)),
        };
        
        match path_result {
            Some(result) => {
                let path_concepts = concepts.into_iter()
                    .filter(|c| result.path.contains(&c.id))
                    .collect::<Vec<_>>();
                
                // Calculate metadata
                let metadata = self.calculate_path_metadata(&result, &path_concepts).await?;
                
                info!("Found path with {} concepts, total cost: {}", path_concepts.len(), result.total_cost);
                
                Ok(PathFindingResult {
                    path: path_concepts,
                    total_cost: result.total_cost,
                    algorithm_used: algorithm.to_string(),
                    metadata,
                })
            }
            None => Err(anyhow!("No path found between {} and {}", request.from_concept, request.to_concept))
        }
    }

    /// Calculate concept similarity scores
    pub async fn calculate_similarity(&self, concept_id: &str, limit: Option<u32>) -> Result<Vec<(Concept, f32)>> {
        info!("Calculating similarity for concept: {}", concept_id);
        
        let base_concept = self.get_concept_by_id(concept_id).await?;
        let concepts = self.load_similar_concepts(&base_concept, limit.unwrap_or(10) as usize).await?;
        
        let mut algorithms = self.algorithms.clone();
        algorithms.add_concepts(concepts.clone());
        
        let similarities = algorithms.traversal_engine()
            .find_similar_concepts(base_concept.id, 0.3, limit.unwrap_or(10) as usize)
            .await?;
        
        let mut results = Vec::new();
        for (similar_id, score) in similarities {
            if let Some(concept) = concepts.iter().find(|c| c.id == similar_id) {
                results.push((concept.clone(), score));
            }
        }
        
        info!("Found {} similar concepts", results.len());
        Ok(results)
    }

    /// Get concept recommendations based on user progress
    pub async fn get_recommendations(&self, user_id: &str, limit: Option<u32>) -> Result<Vec<Concept>> {
        info!("Getting recommendations for user: {}", user_id);
        
        // This would typically query user progress and preferences
        // For now, implement a basic recommendation based on popular concepts
        let popular_concepts = self.get_popular_concepts(limit.unwrap_or(10) as usize).await?;
        
        info!("Generated {} recommendations", popular_concepts.len());
        Ok(popular_concepts)
    }

    /// Private helper methods
    
    async fn get_concept_by_id(&self, concept_id: &str) -> Result<Concept> {
        let params = QueryParameters {
            concept_id: Some(concept_id.to_string()),
            ..Default::default()
        };
        
        let query = self.query_builder.build_query(QueryType::GetConcept, params)?;
        let result = self.client.query(&query).await?;
        
        // Parse the result and extract concept
        self.parse_concept_from_result(result)
    }
    
    async fn get_concept_by_name(&self, name: &str) -> Result<Concept> {
        let params = QueryParameters {
            concept_name: Some(name.to_string()),
            limit: Some(1),
            ..Default::default()
        };
        
        let query = self.query_builder.build_query(QueryType::SearchConcepts, params)?;
        let result = self.client.query(&query).await?;
        
        // Parse the first result
        self.parse_concept_from_search_result(result)
    }
    
    async fn load_related_concepts(&self, concept_id: &Uuid, max_depth: usize) -> Result<Vec<Concept>> {
        // Build a GraphQL query to load concepts within max_depth
        let query = format!(r#"
            query LoadRelatedConcepts($conceptId: ID!) {{
                concept: getConcept(id: $conceptId) {{
                    id
                    name
                    description
                    difficulty
                    category
                    prerequisites {{
                        id
                        name
                        description
                        difficulty
                        category
                    }}
                    enabledBy {{
                        id
                        name
                        description
                        difficulty
                        category
                    }}
                    relatedTo {{
                        id
                        name
                        description
                        difficulty
                        category
                    }}
                }}
            }}
        "#);
        
        let result = self.client.query_with_vars(&query, {
            let mut vars = HashMap::new();
            vars.insert("conceptId".to_string(), concept_id.to_string());
            vars
        }).await?;
        
        self.parse_concepts_from_graph_result(result)
    }
    
    async fn load_path_concepts(&self, from_id: &Uuid, to_id: &Uuid, max_distance: usize) -> Result<Vec<Concept>> {
        // Load concepts that could be part of a path between from and to
        // This is a simplified implementation - in practice, you'd want more sophisticated path loading
        let mut all_concepts = Vec::new();
        
        // Load concepts around the from node
        all_concepts.extend(self.load_related_concepts(from_id, max_distance / 2).await?);
        
        // Load concepts around the to node
        all_concepts.extend(self.load_related_concepts(to_id, max_distance / 2).await?);
        
        // Deduplicate
        all_concepts.sort_by(|a, b| a.id.cmp(&b.id));
        all_concepts.dedup_by(|a, b| a.id == b.id);
        
        Ok(all_concepts)
    }
    
    async fn load_similar_concepts(&self, base_concept: &Concept, limit: usize) -> Result<Vec<Concept>> {
        let params = QueryParameters {
            search_term: Some(base_concept.category.clone()),
            limit: Some(limit as u32 * 2), // Load more to filter better
            constraints: Some(QueryConstraints {
                categories: Some(vec![base_concept.category.clone()]),
                difficulty: vec![base_concept.difficulty.clone()],
                min_quality: 0.5,
                include_subtopics: None,
            }),
            ..Default::default()
        };
        
        let query = self.query_builder.build_query(QueryType::SearchConcepts, params)?;
        let result = self.client.query(&query).await?;
        
        self.parse_concepts_from_search_result(result, Some(limit))
    }
    
    async fn get_popular_concepts(&self, limit: usize) -> Result<Vec<Concept>> {
        // Query for high-quality concepts across categories
        let params = QueryParameters {
            search_term: Some("*".to_string()), // Match all
            limit: Some(limit as u32),
            constraints: Some(QueryConstraints {
                min_quality: 0.7,
                difficulty: vec![], // Any difficulty
                categories: None,
                include_subtopics: None,
            }),
            ..Default::default()
        };
        
        let query = self.query_builder.build_query(QueryType::SearchConcepts, params)?;
        let result = self.client.query(&query).await?;
        
        self.parse_concepts_from_search_result(result, Some(limit))
    }

    async fn build_concept_edges(&self, concepts: &[Concept]) -> Result<Vec<ConceptEdge>> {
        let mut edges = Vec::new();
        
        for concept in concepts {
            // This is a simplified edge building - in practice, you'd query the actual relationships
            // For now, create edges based on category and difficulty relationships
            for other_concept in concepts {
                if concept.id != other_concept.id {
                    if let Some(edge) = self.infer_relationship(concept, other_concept) {
                        edges.push(edge);
                    }
                }
            }
        }
        
        Ok(edges)
    }
    
    async fn build_graph_edges(&self, concepts: &[Concept]) -> Result<Vec<GraphEdge>> {
        let mut edges = Vec::new();
        
        for concept in concepts {
            for other_concept in concepts {
                if concept.id != other_concept.id {
                    if let Some(edge) = self.infer_graph_relationship(concept, other_concept) {
                        edges.push(edge);
                    }
                }
            }
        }
        
        Ok(edges)
    }
    
    fn infer_relationship(&self, from: &Concept, to: &Concept) -> Option<ConceptEdge> {
        // Infer relationship based on concept properties
        let weight = self.calculate_relationship_weight(from, to);
        
        if weight > 0.1 {
            let relationship_type = if from.difficulty == "beginner" && to.difficulty == "intermediate" {
                RelationshipType::Progression
            } else if from.category == to.category {
                RelationshipType::Related
            } else {
                RelationshipType::Similarity
            };
            
            Some(ConceptEdge {
                from: from.id,
                to: to.id,
                relationship: relationship_type,
                weight,
            })
        } else {
            None
        }
    }
    
    fn infer_graph_relationship(&self, from: &Concept, to: &Concept) -> Option<GraphEdge> {
        let weight = self.calculate_relationship_weight(from, to);
        
        if weight > 0.1 {
            let relationship = if from.category == to.category {
                EdgeRelationship::Related
            } else {
                EdgeRelationship::Similarity
            };
            
            Some(GraphEdge {
                from: from.id,
                to: to.id,
                weight,
                relationship,
            })
        } else {
            None
        }
    }
    
    fn calculate_relationship_weight(&self, concept1: &Concept, concept2: &Concept) -> f32 {
        let mut weight = 0.0;
        
        // Category similarity
        if concept1.category == concept2.category {
            weight += 0.4;
        }
        
        // Tag overlap
        let common_tags = concept1.tags.iter()
            .filter(|tag| concept2.tags.contains(tag))
            .count();
        let total_tags = (concept1.tags.len() + concept2.tags.len()).max(1);
        weight += 0.3 * (common_tags as f32 / total_tags as f32);
        
        // Quality difference (closer quality = higher weight)
        let quality_diff = (concept1.quality_score - concept2.quality_score).abs();
        weight += 0.2 * (1.0 - quality_diff);
        
        // Difficulty progression bonus
        if self.is_difficulty_progression(&concept1.difficulty, &concept2.difficulty) {
            weight += 0.1;
        }
        
        weight.min(1.0)
    }
    
    fn is_difficulty_progression(&self, from: &str, to: &str) -> bool {
        matches!(
            (from, to),
            ("beginner", "intermediate") | ("intermediate", "advanced") | ("advanced", "expert")
        )
    }
    
    async fn analyze_concept_relationship(&self, base: &Concept, target: &Concept, traversal_result: &crate::algorithms::traversal::TraversalResult) -> Result<ConceptRelationship> {
        let relationship_type = if base.category == target.category {
            "related"
        } else {
            "similar"
        }.to_string();
        
        let strength = self.calculate_relationship_weight(base, target);
        let path_length = traversal_result.distances.get(&target.id).copied().unwrap_or(0) as u32;
        
        let shared_properties = self.find_shared_properties(base, target);
        
        Ok(ConceptRelationship {
            target_concept: target.clone(),
            relationship_type,
            strength,
            path_length,
            shared_properties,
        })
    }
    
    fn find_shared_properties(&self, concept1: &Concept, concept2: &Concept) -> Vec<String> {
        let mut shared = Vec::new();
        
        if concept1.category == concept2.category {
            shared.push(format!("category:{}", concept1.category));
        }
        
        if concept1.difficulty == concept2.difficulty {
            shared.push(format!("difficulty:{}", concept1.difficulty));
        }
        
        for tag in &concept1.tags {
            if concept2.tags.contains(tag) {
                shared.push(format!("tag:{}", tag));
            }
        }
        
        shared
    }
    
    fn extract_path_from_traversal(&self, traversal: crate::algorithms::traversal::TraversalResult, from: Uuid, to: Uuid) -> Result<Option<crate::algorithms::shortest_path::PathResult>> {
        // Extract path from BFS traversal result
        if !traversal.visited_concepts.contains(&to) {
            return Ok(None);
        }
        
        // Reconstruct path using the path tree
        let mut path = Vec::new();
        let mut current = to;
        path.push(current);
        
        // Trace back through parents (this is simplified - real implementation would need parent tracking)
        for (parent, children) in &traversal.path_tree {
            if children.contains(&current) {
                path.push(*parent);
                current = *parent;
                if current == from {
                    break;
                }
            }
        }
        
        path.reverse();
        
        // Calculate total cost (simplified)
        let total_cost = path.len() as f32;
        
        Ok(Some(crate::algorithms::shortest_path::PathResult {
            path,
            total_cost,
            concepts: traversal.concepts,
        }))
    }
    
    async fn calculate_path_metadata(&self, path_result: &crate::algorithms::shortest_path::PathResult, concepts: &[Concept]) -> Result<PathMetadata> {
        let difficulty_progression = concepts.iter()
            .map(|c| c.difficulty.clone())
            .collect();
        
        let estimated_time = concepts.iter()
            .map(|c| c.estimated_time.unwrap_or(60.0))
            .sum();
        
        // Find potential bottlenecks (concepts with high difficulty or low quality)
        let bottlenecks = concepts.iter()
            .filter(|c| c.difficulty == "expert" || c.quality_score < 0.5)
            .map(|c| c.id)
            .collect();
        
        Ok(PathMetadata {
            alternative_paths: None, // Could be calculated with alternative path algorithms
            bottlenecks,
            difficulty_progression,
            estimated_time,
        })
    }
    
    fn parse_relationship_types(&self, types: Option<Vec<String>>) -> Vec<RelationshipType> {
        match types {
            Some(types) => types.into_iter()
                .filter_map(|t| match t.as_str() {
                    "prerequisite" => Some(RelationshipType::Prerequisite),
                    "progression" => Some(RelationshipType::Progression),
                    "similarity" => Some(RelationshipType::Similarity),
                    "related" => Some(RelationshipType::Related),
                    _ => None,
                })
                .collect(),
            None => vec![RelationshipType::Related, RelationshipType::Prerequisite, RelationshipType::Progression],
        }
    }
    
    // Result parsing methods
    fn parse_concept_from_result(&self, result: serde_json::Value) -> Result<Concept> {
        self.response_parser.parse_concept_from_result(result)
            .map_err(|e| anyhow::Error::from(e))
    }
    
    fn parse_concept_from_search_result(&self, result: serde_json::Value) -> Result<Concept> {
        let concepts = self.response_parser.parse_concepts_from_search_result(result, Some(1))?;
        concepts.into_iter().next()
            .ok_or_else(|| anyhow!("No concepts found in search result"))
    }
    
    fn parse_concepts_from_graph_result(&self, result: serde_json::Value) -> Result<Vec<Concept>> {
        self.response_parser.parse_concepts_from_graph_result(result)
            .map_err(|e| anyhow::Error::from(e))
    }
    
    fn parse_concepts_from_search_result(&self, result: serde_json::Value, limit: Option<usize>) -> Result<Vec<Concept>> {
        self.response_parser.parse_concepts_from_search_result(result, limit)
            .map_err(|e| anyhow::Error::from(e))
    }
}

impl Default for RelationshipDiscoveryRequest {
    fn default() -> Self {
        Self {
            concept_id: String::new(),
            max_depth: Some(3),
            relationship_types: None,
            min_strength: Some(0.3),
            limit: Some(10),
        }
    }
}

impl Default for PathFindingRequest {
    fn default() -> Self {
        Self {
            from_concept: String::new(),
            to_concept: String::new(),
            algorithm: Some("dijkstra".to_string()),
            max_cost: None,
            constraints: None,
        }
    }
}