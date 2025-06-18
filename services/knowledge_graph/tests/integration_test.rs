//! Integration tests for Knowledge Graph Service
//! 
//! These tests verify the end-to-end functionality of the knowledge graph service,
//! including Dgraph integration, algorithms, and API endpoints.

use knowledge_graph::{
    algorithms::{
        shortest_path::{GraphEdge, EdgeRelationship, ShortestPath},
        traversal::{GraphTraversal, TraversalConfig, ConceptEdge, RelationshipType},
        ranking::{ConceptRanking, RankingConfig, RankingEdge},
    },
    client::{DgraphClient, ConnectionConfig},
    graph::{Concept, DgraphConfig, GraphDatabase},
    query::{QueryBuilder, QueryType, QueryParameters, QueryConstraints},
    service::{KnowledgeGraphService, RelationshipDiscoveryRequest, PathFindingRequest},
};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Test data creation helpers
fn create_test_concept(name: &str, category: &str, difficulty: &str) -> Concept {
    Concept {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: Some(format!("Test concept: {}", name)),
        difficulty: difficulty.to_string(),
        category: category.to_string(),
        subcategory: Some("test".to_string()),
        tags: vec!["test".to_string(), category.to_string()],
        quality_score: 0.8,
        estimated_time: Some(60.0),
        embeddings: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        version: 1,
    }
}

#[tokio::test]
async fn test_query_builder_functionality() {
    let query_builder = QueryBuilder::new();
    
    // Test search query building
    let params = QueryParameters {
        search_term: Some("machine learning".to_string()),
        limit: Some(10),
        offset: Some(0),
        constraints: Some(QueryConstraints {
            difficulty: vec!["intermediate".to_string()],
            min_quality: 0.7,
            categories: Some(vec!["computer_science".to_string()]),
            include_subtopics: Some(true),
        }),
        sort_by: Some("qualityScore".to_string()),
        sort_order: Some("desc".to_string()),
        ..Default::default()
    };
    
    let query = query_builder.build_query(QueryType::SearchConcepts, params);
    assert!(query.is_ok());
    let query_string = query.unwrap();
    assert!(query_string.contains("machine learning"));
    assert!(query_string.contains("intermediate"));
    assert!(query_string.contains("qualityScore"));
}

#[tokio::test]
async fn test_concept_creation_query() {
    let query_builder = QueryBuilder::new();
    
    let concept_data = serde_json::json!({
        "name": "Rust Programming",
        "description": "Systems programming language focused on safety and performance",
        "difficulty": "intermediate",
        "category": "programming",
        "tags": ["systems", "memory-safe", "performance"],
        "qualityScore": 0.9,
        "estimatedTime": 120.0
    });
    
    let params = QueryParameters {
        concept_data: Some(concept_data),
        ..Default::default()
    };
    
    let mutation = query_builder.build_query(QueryType::CreateConcept, params);
    assert!(mutation.is_ok());
    let mutation_string = mutation.unwrap();
    assert!(mutation_string.contains("addConcept"));
    assert!(mutation_string.contains("Rust Programming"));
    assert!(mutation_string.contains("intermediate"));
}

#[tokio::test]
async fn test_shortest_path_algorithm() {
    let mut shortest_path = ShortestPath::new();
    
    // Create test concepts
    let concept1 = create_test_concept("Variables", "programming", "beginner");
    let concept2 = create_test_concept("Functions", "programming", "beginner");
    let concept3 = create_test_concept("Structs", "programming", "intermediate");
    let concept4 = create_test_concept("Traits", "programming", "advanced");
    
    let id1 = concept1.id;
    let id2 = concept2.id;
    let id3 = concept3.id;
    let id4 = concept4.id;
    
    // Add concepts to the algorithm
    shortest_path.add_concepts(vec![concept1, concept2, concept3, concept4]);
    
    // Create edges representing learning progression
    shortest_path.add_edges(vec![
        GraphEdge {
            from: id1,
            to: id2,
            weight: 1.0,
            relationship: EdgeRelationship::Progression,
        },
        GraphEdge {
            from: id2,
            to: id3,
            weight: 2.0,
            relationship: EdgeRelationship::Progression,
        },
        GraphEdge {
            from: id3,
            to: id4,
            weight: 3.0,
            relationship: EdgeRelationship::Progression,
        },
        // Alternative path with higher cost
        GraphEdge {
            from: id1,
            to: id4,
            weight: 10.0,
            relationship: EdgeRelationship::Related,
        },
    ]);
    
    // Find shortest path
    let result = shortest_path.dijkstra_path(id1, id4, None).await.unwrap();
    assert!(result.is_some());
    
    let path_result = result.unwrap();
    assert_eq!(path_result.path, vec![id1, id2, id3, id4]);
    assert_eq!(path_result.total_cost, 6.0); // 1 + 2 + 3
    assert_eq!(path_result.concepts.len(), 4);
}

#[tokio::test]
async fn test_astar_algorithm() {
    let mut shortest_path = ShortestPath::new();
    
    let concept1 = create_test_concept("Start", "math", "beginner");
    let concept2 = create_test_concept("Goal", "math", "advanced");
    
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
async fn test_graph_traversal_bfs() {
    let mut traversal = GraphTraversal::new();
    
    // Create a small graph
    let concept1 = create_test_concept("Root", "math", "beginner");
    let concept2 = create_test_concept("Level1a", "math", "intermediate");
    let concept3 = create_test_concept("Level1b", "math", "intermediate");
    let concept4 = create_test_concept("Level2", "math", "advanced");
    
    let id1 = concept1.id;
    let id2 = concept2.id;
    let id3 = concept3.id;
    let id4 = concept4.id;
    
    traversal.add_concepts(vec![concept1, concept2, concept3, concept4]);
    traversal.add_edges(vec![
        ConceptEdge { from: id1, to: id2, relationship: RelationshipType::Progression, weight: 1.0 },
        ConceptEdge { from: id1, to: id3, relationship: RelationshipType::Progression, weight: 1.0 },
        ConceptEdge { from: id2, to: id4, relationship: RelationshipType::Progression, weight: 1.0 },
        ConceptEdge { from: id3, to: id4, relationship: RelationshipType::Progression, weight: 1.0 },
    ]);
    
    let config = TraversalConfig {
        max_depth: Some(3),
        max_nodes: Some(10),
        ..Default::default()
    };
    
    let result = traversal.bfs_traversal(id1, config).await.unwrap();
    
    // Should visit all nodes
    assert_eq!(result.visited_concepts.len(), 4);
    assert!(result.visited_concepts.contains(&id1));
    assert!(result.visited_concepts.contains(&id2));
    assert!(result.visited_concepts.contains(&id3));
    assert!(result.visited_concepts.contains(&id4));
    
    // Check distances
    assert_eq!(result.distances[&id1], 0);
    assert_eq!(result.distances[&id2], 1);
    assert_eq!(result.distances[&id3], 1);
    assert_eq!(result.distances[&id4], 2);
}

#[tokio::test]
async fn test_graph_traversal_dfs() {
    let mut traversal = GraphTraversal::new();
    
    let concept1 = create_test_concept("Start", "science", "beginner");
    let concept2 = create_test_concept("Middle", "science", "intermediate");
    let concept3 = create_test_concept("End", "science", "advanced");
    
    let id1 = concept1.id;
    let id2 = concept2.id;
    let id3 = concept3.id;
    
    traversal.add_concepts(vec![concept1, concept2, concept3]);
    traversal.add_edges(vec![
        ConceptEdge { from: id1, to: id2, relationship: RelationshipType::Progression, weight: 1.0 },
        ConceptEdge { from: id2, to: id3, relationship: RelationshipType::Progression, weight: 1.0 },
    ]);
    
    let config = TraversalConfig::default();
    let result = traversal.dfs_traversal(id1, config).await.unwrap();
    
    assert_eq!(result.visited_concepts.len(), 3);
    assert!(result.path_tree.contains_key(&id1));
}

#[tokio::test]
async fn test_depth_limited_search() {
    let mut traversal = GraphTraversal::new();
    
    let concept1 = create_test_concept("Start", "math", "beginner");
    let concept2 = create_test_concept("Middle", "math", "intermediate");
    let concept3 = create_test_concept("Goal", "math", "advanced");
    
    let id1 = concept1.id;
    let id2 = concept2.id;
    let id3 = concept3.id;
    
    traversal.add_concepts(vec![concept1, concept2, concept3]);
    traversal.add_edges(vec![
        ConceptEdge { from: id1, to: id2, relationship: RelationshipType::Related, weight: 1.0 },
        ConceptEdge { from: id2, to: id3, relationship: RelationshipType::Related, weight: 1.0 },
    ]);
    
    let config = TraversalConfig::default();
    
    // Should find path with sufficient depth
    let result = traversal.depth_limited_search(id1, id3, 3, config.clone()).await.unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap(), vec![id1, id2, id3]);
    
    // Should not find path with insufficient depth
    let result = traversal.depth_limited_search(id1, id3, 1, config).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_concept_ranking() {
    let mut ranking = ConceptRanking::new();
    
    let concept1 = create_test_concept("Hub", "programming", "intermediate");
    let concept2 = create_test_concept("Spoke1", "programming", "beginner");
    let concept3 = create_test_concept("Spoke2", "programming", "beginner");
    let concept4 = create_test_concept("Leaf", "programming", "advanced");
    
    let id1 = concept1.id;
    let id2 = concept2.id;
    let id3 = concept3.id;
    let id4 = concept4.id;
    
    ranking.add_concepts(vec![concept1, concept2, concept3, concept4]);
    
    // Create edges where concept1 is a hub
    ranking.add_edges(vec![
        RankingEdge { from: id2, to: id1, weight: 1.0 },
        RankingEdge { from: id3, to: id1, weight: 1.0 },
        RankingEdge { from: id1, to: id4, weight: 1.0 },
    ]);
    
    let config = RankingConfig {
        max_iterations: 10,
        tolerance: 1e-3,
        ..Default::default()
    };
    
    let result = ranking.pagerank(config).await.unwrap();
    
    // Hub concept should have higher rank
    let hub_rank = result.rankings[&id1];
    let spoke_rank = result.rankings[&id2];
    assert!(hub_rank > spoke_rank);
    
    assert_eq!(result.metadata.algorithm, "PageRank");
    assert!(result.metadata.iterations > 0);
    assert!(result.metadata.total_concepts == 4);
}

#[tokio::test]
async fn test_similarity_calculation() {
    let traversal = GraphTraversal::new();
    
    let concept1 = create_test_concept("Algebra", "math", "intermediate");
    let concept2 = create_test_concept("Calculus", "math", "advanced");
    let concept3 = create_test_concept("History", "social_studies", "beginner");
    
    // Same category concepts should be more similar
    let sim1_2 = traversal.calculate_concept_similarity(&concept1, &concept2);
    let sim1_3 = traversal.calculate_concept_similarity(&concept1, &concept3);
    
    assert!(sim1_2 > sim1_3);
    assert!(sim1_2 >= 0.4); // At least category match bonus
}

#[tokio::test]
async fn test_filter_by_category() {
    let mut traversal = GraphTraversal::new();
    
    let concept1 = create_test_concept("Math1", "math", "beginner");
    let concept2 = create_test_concept("Science1", "science", "beginner");
    let concept3 = create_test_concept("Math2", "math", "intermediate");
    
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

#[tokio::test]
async fn test_service_request_structures() {
    // Test that our service request structures can be created and serialized
    let relationship_request = RelationshipDiscoveryRequest {
        concept_id: "test-concept".to_string(),
        max_depth: Some(3),
        relationship_types: Some(vec!["related".to_string(), "prerequisite".to_string()]),
        min_strength: Some(0.5),
        limit: Some(10),
    };
    
    let serialized = serde_json::to_string(&relationship_request).unwrap();
    assert!(serialized.contains("test-concept"));
    assert!(serialized.contains("related"));
    
    let path_request = PathFindingRequest {
        from_concept: "concept-a".to_string(),
        to_concept: "concept-b".to_string(),
        algorithm: Some("dijkstra".to_string()),
        max_cost: Some(10.0),
        constraints: None,
    };
    
    let serialized = serde_json::to_string(&path_request).unwrap();
    assert!(serialized.contains("concept-a"));
    assert!(serialized.contains("dijkstra"));
}

#[tokio::test]
async fn test_alternative_paths() {
    let mut shortest_path = ShortestPath::new();
    
    // Create a diamond-shaped graph with two paths
    let concept1 = create_test_concept("Start", "programming", "beginner");
    let concept2 = create_test_concept("Path1Mid", "programming", "intermediate");
    let concept3 = create_test_concept("Path2Mid", "programming", "intermediate");
    let concept4 = create_test_concept("Goal", "programming", "advanced");
    
    let id1 = concept1.id;
    let id2 = concept2.id;
    let id3 = concept3.id;
    let id4 = concept4.id;
    
    shortest_path.add_concepts(vec![concept1, concept2, concept3, concept4]);
    shortest_path.add_edges(vec![
        // First path (cheaper)
        GraphEdge { from: id1, to: id2, weight: 1.0, relationship: EdgeRelationship::Prerequisite },
        GraphEdge { from: id2, to: id4, weight: 1.0, relationship: EdgeRelationship::Progression },
        // Second path (more expensive)
        GraphEdge { from: id1, to: id3, weight: 2.0, relationship: EdgeRelationship::Prerequisite },
        GraphEdge { from: id3, to: id4, weight: 1.0, relationship: EdgeRelationship::Progression },
    ]);
    
    let paths = shortest_path.find_alternative_paths(id1, id4, 2, None).await.unwrap();
    
    assert_eq!(paths.len(), 2);
    // First path should be cheaper
    assert!(paths[0].total_cost <= paths[1].total_cost);
    assert_eq!(paths[0].total_cost, 2.0); // 1 + 1
    assert_eq!(paths[1].total_cost, 3.0); // 2 + 1
}

#[tokio::test]
async fn test_connected_components() {
    let mut traversal = GraphTraversal::new();
    
    // Create two disconnected components
    let concept1 = create_test_concept("Group1A", "math", "beginner");
    let concept2 = create_test_concept("Group1B", "math", "intermediate");
    let concept3 = create_test_concept("Group2A", "science", "beginner");
    let concept4 = create_test_concept("Group2B", "science", "intermediate");
    
    let id1 = concept1.id;
    let id2 = concept2.id;
    let id3 = concept3.id;
    let id4 = concept4.id;
    
    traversal.add_concepts(vec![concept1, concept2, concept3, concept4]);
    traversal.add_edges(vec![
        // First component
        ConceptEdge { from: id1, to: id2, relationship: RelationshipType::Related, weight: 1.0 },
        // Second component (disconnected)
        ConceptEdge { from: id3, to: id4, relationship: RelationshipType::Related, weight: 1.0 },
    ]);
    
    let components = traversal.find_connected_components().await.unwrap();
    
    assert_eq!(components.len(), 2);
    // Each component should have 2 concepts
    assert!(components.iter().any(|comp| comp.len() == 2 && comp.contains(&id1) && comp.contains(&id2)));
    assert!(components.iter().any(|comp| comp.len() == 2 && comp.contains(&id3) && comp.contains(&id4)));
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    // Note: These tests would require a running Dgraph instance
    // They are marked as ignored by default
    
    #[tokio::test]
    #[ignore]
    async fn test_dgraph_client_creation() {
        let client = DgraphClient::new("localhost:9080".to_string()).await;
        assert!(client.is_ok());
        
        let client = client.unwrap();
        let health = client.health_check().await;
        assert!(health.is_ok());
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_graph_database_connection() {
        let config = DgraphConfig::default();
        let db = GraphDatabase::new(config).await;
        assert!(db.is_ok());
        
        let db = db.unwrap();
        let is_healthy = db.health_check().await.unwrap();
        assert!(is_healthy);
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_knowledge_graph_service() {
        let config = DgraphConfig::default();
        let service = KnowledgeGraphService::new(config).await;
        assert!(service.is_ok());
        
        // Test would continue with actual service operations...
    }
}

/// Performance benchmarks
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn benchmark_shortest_path_performance() {
        let mut shortest_path = ShortestPath::new();
        
        // Create a larger graph for performance testing
        let mut concepts = Vec::new();
        let mut edges = Vec::new();
        
        const GRAPH_SIZE: usize = 100;
        
        // Create concepts
        for i in 0..GRAPH_SIZE {
            let concept = create_test_concept(
                &format!("Concept{}", i),
                "test",
                if i < 30 { "beginner" } else if i < 70 { "intermediate" } else { "advanced" }
            );
            concepts.push(concept);
        }
        
        // Create edges (chain + some cross-connections)
        for i in 0..GRAPH_SIZE-1 {
            edges.push(GraphEdge {
                from: concepts[i].id,
                to: concepts[i+1].id,
                weight: 1.0,
                relationship: EdgeRelationship::Progression,
            });
            
            // Add some cross-connections
            if i % 10 == 0 && i + 10 < GRAPH_SIZE {
                edges.push(GraphEdge {
                    from: concepts[i].id,
                    to: concepts[i+10].id,
                    weight: 2.0,
                    relationship: EdgeRelationship::Related,
                });
            }
        }
        
        shortest_path.add_concepts(concepts.clone());
        shortest_path.add_edges(edges);
        
        let start = Instant::now();
        let result = shortest_path.dijkstra_path(concepts[0].id, concepts[GRAPH_SIZE-1].id, None).await.unwrap();
        let duration = start.elapsed();
        
        assert!(result.is_some());
        println!("Shortest path on {} nodes took: {:?}", GRAPH_SIZE, duration);
        
        // Should complete within reasonable time
        assert!(duration.as_millis() < 1000); // Less than 1 second
    }
    
    #[tokio::test]
    async fn benchmark_bfs_traversal_performance() {
        let mut traversal = GraphTraversal::new();
        
        const GRAPH_SIZE: usize = 200;
        let mut concepts = Vec::new();
        let mut edges = Vec::new();
        
        // Create a more connected graph
        for i in 0..GRAPH_SIZE {
            let concept = create_test_concept(&format!("Node{}", i), "test", "intermediate");
            concepts.push(concept);
        }
        
        // Create a more densely connected graph
        for i in 0..GRAPH_SIZE {
            for j in 1..=3 {
                if i + j < GRAPH_SIZE {
                    edges.push(ConceptEdge {
                        from: concepts[i].id,
                        to: concepts[i+j].id,
                        relationship: RelationshipType::Related,
                        weight: 1.0,
                    });
                }
            }
        }
        
        traversal.add_concepts(concepts.clone());
        traversal.add_edges(edges);
        
        let config = TraversalConfig {
            max_depth: Some(10),
            max_nodes: Some(GRAPH_SIZE),
            ..Default::default()
        };
        
        let start = Instant::now();
        let result = traversal.bfs_traversal(concepts[0].id, config).await.unwrap();
        let duration = start.elapsed();
        
        println!("BFS traversal on {} nodes found {} concepts in {:?}", 
                 GRAPH_SIZE, result.visited_concepts.len(), duration);
        
        assert!(result.visited_concepts.len() > 10); // Should find many connected concepts
        assert!(duration.as_millis() < 500); // Should be fast
    }
}