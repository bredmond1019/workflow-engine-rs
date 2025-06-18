// =============================================================================
// Node Registry - Maps TypeIds to actual node instances
// =============================================================================

use std::{any::TypeId, collections::HashMap};

use super::Node;

#[derive(Debug)]
pub struct NodeRegistry {
    nodes: HashMap<TypeId, Box<dyn Node>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn register<T: Node + 'static>(&mut self, node: T) {
        self.nodes.insert(TypeId::of::<T>(), Box::new(node));
    }

    pub fn get(&self, type_id: &TypeId) -> Option<&dyn Node> {
        self.nodes.get(type_id).map(|boxed| boxed.as_ref())
    }

    pub fn get_all_node_types(&self) -> Vec<TypeId> {
        self.nodes.keys().copied().collect()
    }

    pub fn get_node_count(&self) -> usize {
        self.nodes.len()
    }
}
