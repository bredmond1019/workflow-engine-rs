//! Graph algorithms module
//! 
//! Contains implementations of various graph algorithms for knowledge
//! graph analysis and learning path optimization.

pub mod shortest_path;
pub mod topological_sort;
pub mod traversal;
pub mod ranking;

pub use shortest_path::*;
pub use topological_sort::*;
pub use traversal::*;
pub use ranking::*;

use crate::graph::Concept;
use anyhow::Result;
use uuid::Uuid;

/// Main algorithms coordinator
#[derive(Clone)]
pub struct GraphAlgorithms {
    shortest_path: shortest_path::ShortestPath,
    topological_sort: topological_sort::TopologicalSort,
    traversal: traversal::GraphTraversal,
    ranking: ranking::ConceptRanking,
}

impl GraphAlgorithms {
    /// Create a new graph algorithms coordinator
    pub fn new() -> Self {
        Self {
            shortest_path: shortest_path::ShortestPath::new(),
            topological_sort: topological_sort::TopologicalSort::new(),
            traversal: traversal::GraphTraversal::new(),
            ranking: ranking::ConceptRanking::new(),
        }
    }

    /// Add concepts to all algorithms
    pub fn add_concepts(&mut self, concepts: Vec<Concept>) {
        self.shortest_path.add_concepts(concepts.clone());
        self.topological_sort.add_concepts(concepts.clone());
        self.traversal.add_concepts(concepts.clone());
        self.ranking.add_concepts(concepts);
    }

    /// Find shortest path between concepts
    pub async fn shortest_path(
        &self,
        from: Uuid,
        to: Uuid,
        max_cost: Option<f32>,
    ) -> Result<Option<shortest_path::PathResult>> {
        self.shortest_path.dijkstra_path(from, to, max_cost).await
    }

    /// Sort concepts topologically
    pub async fn topological_sort(
        &self,
        concept_ids: Option<Vec<Uuid>>,
    ) -> Result<topological_sort::TopologicalResult> {
        self.topological_sort.kahn_sort(concept_ids).await
    }

    /// Perform BFS traversal
    pub async fn bfs_traversal(
        &self,
        start: Uuid,
        config: traversal::TraversalConfig,
    ) -> Result<traversal::TraversalResult> {
        self.traversal.bfs_traversal(start, config).await
    }

    /// Calculate PageRank scores
    pub async fn pagerank(
        &self,
        config: ranking::RankingConfig,
    ) -> Result<ranking::RankingResult> {
        self.ranking.pagerank(config).await
    }

    /// Get access to individual algorithm implementations
    pub fn shortest_path_engine(&self) -> &shortest_path::ShortestPath {
        &self.shortest_path
    }

    pub fn topological_sort_engine(&self) -> &topological_sort::TopologicalSort {
        &self.topological_sort
    }

    pub fn traversal_engine(&self) -> &traversal::GraphTraversal {
        &self.traversal
    }

    pub fn ranking_engine(&self) -> &ranking::ConceptRanking {
        &self.ranking
    }

    /// Get mutable access to individual algorithm implementations
    pub fn shortest_path_engine_mut(&mut self) -> &mut shortest_path::ShortestPath {
        &mut self.shortest_path
    }

    pub fn topological_sort_engine_mut(&mut self) -> &mut topological_sort::TopologicalSort {
        &mut self.topological_sort
    }

    pub fn traversal_engine_mut(&mut self) -> &mut traversal::GraphTraversal {
        &mut self.traversal
    }

    pub fn ranking_engine_mut(&mut self) -> &mut ranking::ConceptRanking {
        &mut self.ranking
    }
}

impl Default for GraphAlgorithms {
    fn default() -> Self {
        Self::new()
    }
}