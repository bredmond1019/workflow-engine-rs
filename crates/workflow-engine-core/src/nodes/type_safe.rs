//! Type-safe node identification and workflow builder utilities
//! 
//! This module provides type-safe alternatives to TypeId for node
//! identification and workflow construction.

use std::marker::PhantomData;
use std::fmt::Debug;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use super::{Node, AsyncNode};
use crate::error::WorkflowError;
use crate::task::TaskContext;

/// Type-safe node identifier using phantom types
/// 
/// This replaces the use of TypeId with a more type-safe approach
/// that provides better compile-time guarantees.
#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId<T> {
    id: Uuid,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> Clone for NodeId<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

impl<T> Copy for NodeId<T> {}

impl<T> NodeId<T> {
    /// Create a new type-safe node ID
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            _phantom: PhantomData,
        }
    }
    
    /// Create a node ID from an existing UUID
    pub fn from_uuid(id: Uuid) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }
    
    /// Get the underlying UUID
    pub fn uuid(&self) -> Uuid {
        self.id
    }
    
    /// Convert to a string representation
    pub fn to_string(&self) -> String {
        self.id.to_string()
    }
}

impl<T> Default for NodeId<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::fmt::Display for NodeId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Type-safe node configuration
/// 
/// Replaces the use of raw TypeId with typed node references
#[derive(Debug, Clone)]
pub struct TypedNodeConfig<T: Node> {
    pub id: NodeId<T>,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub config_data: serde_json::Value,
    _phantom: PhantomData<T>,
}

impl<T: Node> TypedNodeConfig<T> {
    /// Create a new typed node configuration
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: NodeId::new(),
            name: name.into(),
            description: description.into(),
            enabled: true,
            timeout_seconds: 30,
            retry_attempts: 3,
            config_data: serde_json::Value::Null,
            _phantom: PhantomData,
        }
    }
    
    /// Set the node as enabled or disabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    /// Set the timeout in seconds
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }
    
    /// Set the number of retry attempts
    pub fn retries(mut self, attempts: u32) -> Self {
        self.retry_attempts = attempts;
        self
    }
    
    /// Set configuration data
    pub fn config(mut self, data: serde_json::Value) -> Self {
        self.config_data = data;
        self
    }
}

/// Type-safe workflow builder with compile-time validation
/// 
/// This builder provides a fluent API for constructing workflows
/// with type-safe node connections and validation.
#[derive(Debug)]
pub struct TypedWorkflowBuilder {
    name: String,
    description: String,
    nodes: Vec<Box<dyn ErasedNodeConfig>>,
    connections: Vec<Connection>,
}

/// Erased node configuration for storage in collections
trait ErasedNodeConfig: Debug + Send + Sync {
    fn id(&self) -> Uuid;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn is_enabled(&self) -> bool;
}

impl<T: Node + 'static> ErasedNodeConfig for TypedNodeConfig<T> {
    fn id(&self) -> Uuid {
        self.id.uuid()
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Connection between two nodes in a workflow
#[derive(Debug, Clone)]
pub struct Connection {
    from: Uuid,
    to: Uuid,
    condition: Option<String>,
}

impl TypedWorkflowBuilder {
    /// Create a new typed workflow builder
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            nodes: Vec::new(),
            connections: Vec::new(),
        }
    }
    
    /// Add a node to the workflow
    pub fn add_node<T: Node + 'static>(mut self, config: TypedNodeConfig<T>) -> Self {
        self.nodes.push(Box::new(config));
        self
    }
    
    /// Connect two nodes in the workflow
    pub fn connect<T: Node, U: Node>(
        mut self, 
        from: NodeId<T>, 
        to: NodeId<U>
    ) -> Self {
        self.connections.push(Connection {
            from: from.uuid(),
            to: to.uuid(),
            condition: None,
        });
        self
    }
    
    /// Connect two nodes with a condition
    pub fn connect_if<T: Node, U: Node>(
        mut self, 
        from: NodeId<T>, 
        to: NodeId<U>,
        condition: impl Into<String>
    ) -> Self {
        self.connections.push(Connection {
            from: from.uuid(),
            to: to.uuid(),
            condition: Some(condition.into()),
        });
        self
    }
    
    /// Validate the workflow configuration
    pub fn validate(&self) -> Result<(), WorkflowError> {
        // Check for cycles
        if self.has_cycles() {
            return Err(WorkflowError::ValidationError {
                message: "Workflow contains cycles".to_string()
            });
        }
        
        // Check for orphaned nodes
        if self.has_orphaned_nodes() {
            return Err(WorkflowError::ValidationError {
                message: "Workflow contains orphaned nodes".to_string()
            });
        }
        
        // Check for disabled critical paths
        if self.has_disabled_critical_path() {
            return Err(WorkflowError::ValidationError {
                message: "Critical path contains disabled nodes".to_string()
            });
        }
        
        Ok(())
    }
    
    /// Check if the workflow has cycles
    fn has_cycles(&self) -> bool {
        // Simple cycle detection using DFS
        // In a production implementation, this would be more sophisticated
        false // Placeholder
    }
    
    /// Check if the workflow has orphaned nodes
    fn has_orphaned_nodes(&self) -> bool {
        let connected_nodes: std::collections::HashSet<Uuid> = self.connections
            .iter()
            .flat_map(|conn| [conn.from, conn.to])
            .collect();
            
        self.nodes.iter().any(|node| !connected_nodes.contains(&node.id()))
    }
    
    /// Check if critical path has disabled nodes
    fn has_disabled_critical_path(&self) -> bool {
        // Check if any path from start to end contains disabled nodes
        // This is a simplified check
        false // Placeholder
    }
    
    /// Build the workflow
    pub fn build(self) -> Result<TypedWorkflow, WorkflowError> {
        self.validate()?;
        
        Ok(TypedWorkflow {
            name: self.name,
            description: self.description,
            nodes: self.nodes,
            connections: self.connections,
        })
    }
}

/// Type-safe workflow definition
#[derive(Debug)]
pub struct TypedWorkflow {
    pub name: String,
    pub description: String,
    pub nodes: Vec<Box<dyn ErasedNodeConfig>>,
    pub connections: Vec<Connection>,
}

impl TypedWorkflow {
    /// Get a list of starting nodes (nodes with no incoming connections)
    pub fn start_nodes(&self) -> Vec<Uuid> {
        let has_incoming: std::collections::HashSet<Uuid> = self.connections
            .iter()
            .map(|conn| conn.to)
            .collect();
            
        self.nodes
            .iter()
            .filter(|node| !has_incoming.contains(&node.id()))
            .map(|node| node.id())
            .collect()
    }
    
    /// Get the next nodes to execute after the given node
    pub fn next_nodes(&self, current: Uuid) -> Vec<Uuid> {
        self.connections
            .iter()
            .filter(|conn| conn.from == current)
            .map(|conn| conn.to)
            .collect()
    }
    
    /// Get node configuration by ID
    pub fn get_node(&self, id: Uuid) -> Option<&dyn ErasedNodeConfig> {
        self.nodes
            .iter()
            .find(|node| node.id() == id)
            .map(|node| node.as_ref())
    }
}

/// Convenience methods for common workflow patterns
impl TypedWorkflowBuilder {
    /// Create a simple linear workflow
    pub fn linear<T1, T2, T3>(
        name: impl Into<String>,
        node1: TypedNodeConfig<T1>,
        node2: TypedNodeConfig<T2>, 
        node3: TypedNodeConfig<T3>
    ) -> Self 
    where
        T1: Node + 'static,
        T2: Node + 'static,
        T3: Node + 'static,
    {
        // Get the IDs before moving the nodes
        let id1 = NodeId::<T1>::from_uuid(node1.id());
        let id2 = NodeId::<T2>::from_uuid(node2.id());
        let id3 = NodeId::<T3>::from_uuid(node3.id());
        
        Self::new(name, "Linear workflow")
            .add_node(node1)
            .add_node(node2)
            .add_node(node3)
            .connect(id1, id2.clone())
            .connect(id2, id3)
    }
    
    /// Create a parallel workflow (fan-out then fan-in)
    pub fn parallel<T1, T2, T3, T4>(
        name: impl Into<String>,
        start: TypedNodeConfig<T1>,
        parallel1: TypedNodeConfig<T2>,
        parallel2: TypedNodeConfig<T3>,
        end: TypedNodeConfig<T4>
    ) -> Self 
    where
        T1: Node + 'static,
        T2: Node + 'static,
        T3: Node + 'static,
        T4: Node + 'static,
    {
        // Get the IDs before moving the nodes
        let start_id = NodeId::<T1>::from_uuid(start.id());
        let p1_id = NodeId::<T2>::from_uuid(parallel1.id());
        let p2_id = NodeId::<T3>::from_uuid(parallel2.id());
        let end_id = NodeId::<T4>::from_uuid(end.id());
        
        Self::new(name, "Parallel workflow")
            .add_node(start)
            .add_node(parallel1)
            .add_node(parallel2)
            .add_node(end)
            .connect(start_id.clone(), p1_id.clone())
            .connect(start_id, p2_id.clone())
            .connect(p1_id, end_id.clone())
            .connect(p2_id, end_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::Node;
    
    #[derive(Debug)]
    struct TestNode;
    
    impl Node for TestNode {
        fn process(&self, context: TaskContext) -> Result<TaskContext, WorkflowError> {
            Ok(context)
        }
    }
    
    #[test]
    fn test_node_id_creation() {
        let id1: NodeId<TestNode> = NodeId::new();
        let id2: NodeId<TestNode> = NodeId::new();
        
        assert_ne!(id1, id2);
        assert_ne!(id1.uuid(), id2.uuid());
    }
    
    #[test]
    fn test_typed_node_config() {
        let config = TypedNodeConfig::<TestNode>::new("test", "Test node")
            .enabled(true)
            .timeout(60)
            .retries(5);
            
        assert_eq!(config.name, "test");
        assert_eq!(config.description, "Test node");
        assert!(config.enabled);
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.retry_attempts, 5);
    }
    
    #[test]
    fn test_workflow_builder() {
        let node1 = TypedNodeConfig::<TestNode>::new("node1", "First node");
        let node2 = TypedNodeConfig::<TestNode>::new("node2", "Second node");
        
        let id1 = node1.id;
        let id2 = node2.id;
        
        let workflow = TypedWorkflowBuilder::new("test", "Test workflow")
            .add_node(node1)
            .add_node(node2)
            .connect(id1, id2)
            .build()
            .expect("Failed to build workflow");
            
        assert_eq!(workflow.name, "test");
        assert_eq!(workflow.nodes.len(), 2);
        assert_eq!(workflow.connections.len(), 1);
    }
    
    #[test]
    fn test_linear_workflow() {
        let node1 = TypedNodeConfig::<TestNode>::new("start", "Start node");
        let node2 = TypedNodeConfig::<TestNode>::new("middle", "Middle node");
        let node3 = TypedNodeConfig::<TestNode>::new("end", "End node");
        
        let workflow = TypedWorkflowBuilder::linear("linear_test", node1, node2, node3)
            .build()
            .expect("Failed to build linear workflow");
            
        assert_eq!(workflow.connections.len(), 2);
        assert_eq!(workflow.start_nodes().len(), 1);
    }
}