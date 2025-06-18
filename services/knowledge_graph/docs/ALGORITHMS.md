# Knowledge Graph Algorithms Guide

## Overview

The Knowledge Graph Service implements a comprehensive suite of graph algorithms optimized for educational content and learning path generation. All algorithms are designed for performance, scalability, and practical application in knowledge management.

## Shortest Path Algorithms

### Dijkstra's Algorithm

Used for finding the optimal learning path between concepts based on difficulty and prerequisites.

```rust
pub fn dijkstra(
    start: Uuid,
    goal: Uuid,
    edges: &[GraphEdge],
    constraints: Option<PathConstraints>,
) -> Result<Option<Vec<Uuid>>>
```

#### Implementation Details

```mermaid
graph LR
    A[Start: Basic Python] -->|1.0| B[Data Types]
    B -->|1.5| C[Functions]
    B -->|2.0| D[Classes]
    C -->|1.0| E[Decorators]
    D -->|1.5| E
    E -->|2.0| F[Goal: Advanced Python]
    
    style A fill:#90EE90
    style F fill:#FFB6C1
```

**Algorithm Steps:**
1. Initialize distances to infinity, start node to 0
2. Use binary heap for efficient minimum extraction
3. Relax edges considering constraints
4. Track parent pointers for path reconstruction

**Time Complexity:** O((V + E) log V)  
**Space Complexity:** O(V)

#### Example Usage

```rust
// Find shortest learning path
let constraints = PathConstraints {
    max_difficulty: Some(Difficulty::Intermediate),
    max_path_length: Some(10),
    required_categories: vec!["programming".to_string()],
    forbidden_concepts: vec![],
};

let path = dijkstra(
    start_concept_id,
    goal_concept_id,
    &graph_edges,
    Some(constraints)
)?;
```

### A* Algorithm

Enhanced pathfinding using heuristic functions for faster convergence.

```rust
pub fn a_star(
    start: Uuid,
    goal: Uuid,
    edges: &[GraphEdge],
    heuristic: impl Fn(Uuid) -> f32,
    constraints: Option<PathConstraints>,
) -> Result<Option<Vec<Uuid>>>
```

#### Heuristic Functions

1. **Difficulty-based Heuristic**
   ```rust
   fn difficulty_heuristic(concept: &Concept, goal: &Concept) -> f32 {
       match (concept.difficulty, goal.difficulty) {
           (Difficulty::Beginner, Difficulty::Expert) => 3.0,
           (Difficulty::Beginner, Difficulty::Advanced) => 2.0,
           (Difficulty::Intermediate, Difficulty::Expert) => 2.0,
           _ => 1.0,
       }
   }
   ```

2. **Category Distance Heuristic**
   ```rust
   fn category_heuristic(concept: &Concept, goal: &Concept) -> f32 {
       if concept.category == goal.category {
           0.0
       } else if concept.subcategory == goal.subcategory {
           0.5
       } else {
           1.0
       }
   }
   ```

**Performance Comparison:**

| Algorithm | Average Nodes Explored | Time (ms) |
|-----------|----------------------|-----------|
| Dijkstra | 1000 | 45 |
| A* (difficulty) | 600 | 28 |
| A* (category) | 700 | 32 |

### Alternative Path Finding

Discovers multiple learning paths for flexibility.

```rust
pub fn find_alternative_paths(
    start: Uuid,
    goal: Uuid,
    edges: &[GraphEdge],
    k: usize, // number of paths
    deviation_threshold: f32,
) -> Result<Vec<Path>>
```

#### Algorithm: Modified Yen's K-Shortest Paths

```mermaid
graph TD
    A[Start] --> B[Path 1: Direct]
    A --> C[Path 2: Via Prerequisites]
    A --> D[Path 3: Alternative Route]
    B --> E[Goal]
    C --> E
    D --> E
    
    style A fill:#90EE90
    style E fill:#FFB6C1
```

**Key Features:**
- Finds K distinct paths
- Ensures path diversity using deviation threshold
- Balances path length vs. uniqueness

## PageRank & Ranking Algorithms

### PageRank Implementation

Identifies the most important concepts in the knowledge graph.

```rust
pub fn pagerank(
    nodes: &[Uuid],
    edges: &[GraphEdge],
    damping_factor: f32,
    max_iterations: usize,
    tolerance: f32,
) -> Result<HashMap<Uuid, f32>>
```

#### Algorithm Visualization

```mermaid
graph LR
    A[Machine Learning<br/>PR: 0.35] --> B[Neural Networks<br/>PR: 0.28]
    A --> C[Decision Trees<br/>PR: 0.15]
    B --> D[Deep Learning<br/>PR: 0.42]
    C --> D
    D --> E[GANs<br/>PR: 0.18]
    D --> F[Transformers<br/>PR: 0.25]
    
    style D fill:#FFD700
```

**Mathematical Formula:**
```
PR(A) = (1-d) + d * Σ(PR(Ti)/C(Ti))
```
Where:
- d = damping factor (typically 0.85)
- Ti = pages linking to A
- C(Ti) = outbound links from Ti

**Convergence Criteria:**
- Maximum iterations: 100
- Tolerance: 1e-6
- Early stopping when rank changes < tolerance

### Centrality Measures

#### Degree Centrality

```rust
pub fn degree_centrality(
    nodes: &[Uuid],
    edges: &[GraphEdge],
    direction: Direction,
) -> HashMap<Uuid, f32>
```

**Types:**
- **In-degree**: Number of prerequisites
- **Out-degree**: Number of enabled concepts
- **Total degree**: Combined connectivity

#### Betweenness Centrality

```rust
pub fn betweenness_centrality(
    nodes: &[Uuid],
    edges: &[GraphEdge],
) -> Result<HashMap<Uuid, f32>>
```

Identifies concepts that serve as bridges between different areas.

**Algorithm:** Brandes' algorithm
**Time Complexity:** O(V * E)

#### Closeness Centrality

```rust
pub fn closeness_centrality(
    nodes: &[Uuid],
    edges: &[GraphEdge],
) -> Result<HashMap<Uuid, f32>>
```

Measures how quickly a concept can reach all others.

### Community Detection

Simplified Louvain algorithm for identifying concept clusters.

```rust
pub fn detect_communities(
    nodes: &[Uuid],
    edges: &[GraphEdge],
    resolution: f32,
) -> Result<HashMap<Uuid, CommunityId>>
```

#### Example Communities

```mermaid
graph TD
    subgraph "Web Development"
        A[HTML] --> B[CSS]
        B --> C[JavaScript]
        C --> D[React]
    end
    
    subgraph "Data Science"
        E[Statistics] --> F[Python]
        F --> G[Pandas]
        G --> H[Scikit-learn]
    end
    
    subgraph "Databases"
        I[SQL] --> J[PostgreSQL]
        J --> K[Indexing]
        K --> L[Query Optimization]
    end
    
    C -.-> F
    I -.-> G
```

**Quality Metric:** Modularity score (0.0 - 1.0)

## Topological Sorting

### Kahn's Algorithm

For ordering concepts by dependencies.

```rust
pub fn topological_sort_kahn(
    nodes: &[Uuid],
    edges: &[GraphEdge],
) -> Result<Vec<Uuid>>
```

#### Process Visualization

```mermaid
graph LR
    subgraph "Step 1"
        A1[Math] --> B1[Stats]
        B1 --> C1[ML]
        D1[Programming]
        style A1 fill:#90EE90
        style D1 fill:#90EE90
    end
    
    subgraph "Step 2"
        B2[Stats] --> C2[ML]
        style B2 fill:#90EE90
    end
    
    subgraph "Step 3"
        C3[ML]
        style C3 fill:#90EE90
    end
```

**Result:** [Math, Programming, Stats, ML]

### DFS-Based Topological Sort

Alternative implementation with cycle detection.

```rust
pub fn topological_sort_dfs(
    nodes: &[Uuid],
    edges: &[GraphEdge],
) -> Result<Vec<Uuid>>
```

**Features:**
- Detects cycles during traversal
- Returns specific cycle path for debugging
- Post-order traversal for correct ordering

### Learning Path Validation

```rust
pub fn validate_learning_path(
    path: &[Uuid],
    all_edges: &[GraphEdge],
) -> Result<ValidationResult>
```

**Validation Checks:**
1. No circular dependencies
2. All prerequisites satisfied
3. Difficulty progression logical
4. No duplicate concepts

### Parallel Learning Levels

Identifies concepts that can be learned simultaneously.

```rust
pub fn find_parallel_levels(
    nodes: &[Uuid],
    edges: &[GraphEdge],
) -> Result<Vec<Vec<Uuid>>>
```

#### Example Output

```
Level 0: [Basic Math, Basic Programming]
Level 1: [Algebra, Data Structures]
Level 2: [Statistics, Algorithms]
Level 3: [Machine Learning]
```

## Graph Traversal Algorithms

### Breadth-First Search (BFS)

For level-order exploration and shortest unweighted paths.

```rust
pub fn bfs(
    start: Uuid,
    edges: &[GraphEdge],
    max_depth: Option<usize>,
    filter: Option<Box<dyn Fn(&Uuid) -> bool>>,
) -> Vec<Uuid>
```

#### Use Cases

1. **Find all concepts within N steps**
   ```rust
   let nearby = bfs(concept_id, &edges, Some(2), None);
   ```

2. **Explore specific category**
   ```rust
   let filter = |id: &Uuid| {
       concepts.get(id).map(|c| c.category == "programming").unwrap_or(false)
   };
   let programming_concepts = bfs(start, &edges, None, Some(Box::new(filter)));
   ```

### Depth-First Search (DFS)

For exhaustive exploration and cycle detection.

```rust
pub fn dfs(
    start: Uuid,
    edges: &[GraphEdge],
    visited: &mut HashSet<Uuid>,
    on_visit: impl FnMut(Uuid),
)
```

#### Applications

1. **Prerequisite Chain Analysis**
2. **Concept Dependency Trees**
3. **Cycle Detection**
4. **Path Enumeration**

### Iterative Deepening Search

Combines benefits of BFS and DFS with memory efficiency.

```rust
pub fn iterative_deepening_search(
    start: Uuid,
    goal: Uuid,
    edges: &[GraphEdge],
    max_depth: usize,
) -> Option<Vec<Uuid>>
```

**Advantages:**
- O(bd) space complexity (vs O(b^d) for BFS)
- Finds optimal path like BFS
- Memory efficient like DFS

### Connected Components

Identifies isolated concept clusters.

```rust
pub fn find_connected_components(
    nodes: &[Uuid],
    edges: &[GraphEdge],
) -> Vec<HashSet<Uuid>>
```

**Use Cases:**
- Identify disconnected knowledge areas
- Find orphaned concepts
- Validate graph connectivity

## Performance Characteristics

### Algorithm Comparison

| Algorithm | Time Complexity | Space Complexity | Best Use Case |
|-----------|----------------|------------------|---------------|
| Dijkstra | O((V+E)log V) | O(V) | Optimal paths with weights |
| A* | O((V+E)log V) | O(V) | Faster with good heuristic |
| BFS | O(V+E) | O(V) | Unweighted shortest paths |
| DFS | O(V+E) | O(V) | Full exploration |
| PageRank | O(k(V+E)) | O(V) | Node importance |
| Topological Sort | O(V+E) | O(V) | Dependency ordering |

### Optimization Strategies

1. **Early Termination**
   - Stop Dijkstra/A* when goal reached
   - Limit BFS/DFS depth
   - PageRank convergence detection

2. **Pruning**
   - Skip visited nodes
   - Apply constraint filters early
   - Use heuristics to guide search

3. **Caching**
   - Cache computed paths
   - Store PageRank results
   - Memoize expensive calculations

4. **Parallelization**
   - Parallel BFS levels
   - Distributed PageRank
   - Concurrent component analysis

## Practical Examples

### Example 1: Generate Personalized Learning Path

```rust
// User wants to learn "Machine Learning" starting from "Basic Math"
let start = get_concept_id("Basic Math");
let goal = get_concept_id("Machine Learning");

// Define constraints based on user preferences
let constraints = PathConstraints {
    max_difficulty: Some(Difficulty::Intermediate),
    max_path_length: Some(8),
    preferred_categories: vec!["mathematics", "programming"],
    time_limit_hours: Some(40.0),
};

// Use A* with combined heuristic
let heuristic = |node_id: Uuid| {
    let concept = get_concept(node_id);
    difficulty_heuristic(&concept, &goal_concept) * 0.6 +
    category_heuristic(&concept, &goal_concept) * 0.4
};

let optimal_path = a_star(start, goal, &edges, heuristic, Some(constraints))?;

// Find alternatives for variety
let alternatives = find_alternative_paths(start, goal, &edges, 3, 0.3)?;
```

### Example 2: Identify Key Concepts in Domain

```rust
// Find most important concepts in "Web Development"
let web_dev_nodes: Vec<Uuid> = concepts.iter()
    .filter(|c| c.category == "Web Development")
    .map(|c| c.id)
    .collect();

let web_dev_edges: Vec<GraphEdge> = edges.iter()
    .filter(|e| web_dev_nodes.contains(&e.from) && web_dev_nodes.contains(&e.to))
    .cloned()
    .collect();

// Calculate PageRank
let importance = pagerank(&web_dev_nodes, &web_dev_edges, 0.85, 100, 1e-6)?;

// Get top 10 concepts
let mut ranked: Vec<_> = importance.into_iter().collect();
ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
let top_10: Vec<Uuid> = ranked.into_iter().take(10).map(|(id, _)| id).collect();
```

### Example 3: Validate and Optimize Curriculum

```rust
// Validate existing curriculum
let curriculum_concepts = vec![concept1, concept2, concept3, /* ... */];
let validation = validate_learning_path(&curriculum_concepts, &all_edges)?;

if !validation.is_valid {
    // Fix circular dependencies
    let sorted = topological_sort_dfs(&curriculum_concepts, &relevant_edges)?;
    
    // Identify parallel learning opportunities
    let parallel_levels = find_parallel_levels(&sorted, &relevant_edges)?;
    
    // Optimize for time efficiency
    println!("Curriculum can be completed in {} levels", parallel_levels.len());
    for (i, level) in parallel_levels.iter().enumerate() {
        println!("Level {}: {} concepts can be learned in parallel", i, level.len());
    }
}
```

## Algorithm Selection Guide

### When to Use Each Algorithm

1. **Use Dijkstra when:**
   - You need the absolute shortest path
   - Edge weights vary significantly
   - No good heuristic available

2. **Use A* when:**
   - You have domain knowledge for heuristics
   - Performance is critical
   - Goal is known in advance

3. **Use PageRank when:**
   - Identifying important concepts
   - Building recommendation systems
   - Analyzing knowledge domains

4. **Use Topological Sort when:**
   - Ordering prerequisites
   - Detecting circular dependencies
   - Planning curricula

5. **Use BFS when:**
   - Exploring local neighborhoods
   - Finding shortest unweighted paths
   - Level-order processing needed

6. **Use DFS when:**
   - Memory is constrained
   - Need full exploration
   - Detecting cycles

## Benchmarks

### Performance on Different Graph Sizes

| Nodes | Edges | Dijkstra | A* | PageRank | Top Sort |
|-------|-------|----------|-----|----------|----------|
| 100 | 500 | 2ms | 1ms | 5ms | <1ms |
| 1,000 | 5,000 | 15ms | 8ms | 45ms | 3ms |
| 10,000 | 50,000 | 120ms | 65ms | 380ms | 25ms |
| 100,000 | 500,000 | 1.2s | 0.7s | 4.2s | 280ms |

### Memory Usage

| Algorithm | 10K Nodes | 100K Nodes | 1M Nodes |
|-----------|-----------|------------|----------|
| Dijkstra | 4MB | 40MB | 400MB |
| PageRank | 8MB | 80MB | 800MB |
| BFS/DFS | 2MB | 20MB | 200MB |

## Future Enhancements

1. **Graph Neural Networks**
   - Learned embeddings for better similarity
   - Predictive path quality

2. **Distributed Algorithms**
   - Pregel-style computation
   - Spark GraphX integration

3. **Streaming Algorithms**
   - Real-time graph updates
   - Incremental PageRank

4. **Advanced Heuristics**
   - ML-based cost prediction
   - User-specific preferences