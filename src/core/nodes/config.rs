use std::any::TypeId;

use super::Node;

#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub node_type: TypeId,
    pub connections: Vec<TypeId>,
    pub is_router: bool,
    pub description: Option<String>,
    pub parallel_nodes: Vec<TypeId>,
}

impl NodeConfig {
    pub fn new<T: Node + 'static>() -> Self {
        Self {
            node_type: TypeId::of::<T>(),
            connections: Vec::new(),
            is_router: false,
            description: None,
            parallel_nodes: Vec::new(),
        }
    }

    pub fn with_connections(mut self, connections: Vec<TypeId>) -> Self {
        self.connections = connections;
        self
    }

    pub fn with_router(mut self, is_router: bool) -> Self {
        self.is_router = is_router;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_parallel_nodes(mut self, parallel_nodes: Vec<TypeId>) -> Self {
        self.parallel_nodes = parallel_nodes;
        self
    }
}
