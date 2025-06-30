// =============================================================================
// Workflow Validator
// =============================================================================

use std::{
    any::TypeId,
    collections::{HashSet, VecDeque},
};

use crate::{error::WorkflowError, workflow::schema::WorkflowSchema};

pub struct WorkflowValidator<'a> {
    schema: &'a WorkflowSchema,
}

impl<'a> WorkflowValidator<'a> {
    pub fn new(schema: &'a WorkflowSchema) -> Self {
        Self { schema }
    }

    pub fn validate(&self) -> Result<(), WorkflowError> {
        self.validate_connections()?;
        self.validate_dag()?;
        self.validate_complex_cycles()?;
        Ok(())
    }

    fn validate_dag(&self) -> Result<(), WorkflowError> {
        if self.has_cycle() {
            return Err(WorkflowError::CycleDetected);
        }

        let reachable_nodes = self.get_reachable_nodes();
        let all_nodes: HashSet<TypeId> = self.schema.nodes.iter().map(|nc| nc.node_type).collect();
        let unreachable: Vec<String> = all_nodes
            .difference(&reachable_nodes)
            .map(|id| format!("{:?}", id))
            .collect();

        if !unreachable.is_empty() {
            return Err(WorkflowError::UnreachableNodes { nodes: unreachable });
        }

        Ok(())
    }

    fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        fn dfs(
            node: TypeId,
            schema: &WorkflowSchema,
            visited: &mut HashSet<TypeId>,
            rec_stack: &mut HashSet<TypeId>,
        ) -> bool {
            visited.insert(node);
            rec_stack.insert(node);

            if let Some(node_config) = schema.nodes.iter().find(|nc| nc.node_type == node) {
                for &neighbor in &node_config.connections {
                    if !visited.contains(&neighbor) {
                        if dfs(neighbor, schema, visited, rec_stack) {
                            return true;
                        }
                    } else if rec_stack.contains(&neighbor) {
                        return true;
                    }
                }
            }

            rec_stack.remove(&node);
            false
        }

        for node_config in &self.schema.nodes {
            if !visited.contains(&node_config.node_type) && dfs(
                node_config.node_type,
                self.schema,
                &mut visited,
                &mut rec_stack,
            ) {
                return true;
            }
        }

        false
    }

    fn get_reachable_nodes(&self) -> HashSet<TypeId> {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(self.schema.start);

        while let Some(node) = queue.pop_front() {
            if !reachable.contains(&node) {
                reachable.insert(node);
                if let Some(node_config) = self.schema.nodes.iter().find(|nc| nc.node_type == node)
                {
                    queue.extend(&node_config.connections);
                }
            }
        }

        reachable
    }

    fn validate_connections(&self) -> Result<(), WorkflowError> {
        // Build a set of all node types for quick lookup
        let all_node_types: HashSet<TypeId> = self.schema.nodes.iter().map(|nc| nc.node_type).collect();

        for node_config in &self.schema.nodes {
            // Check router configuration
            if node_config.connections.len() > 1 && !node_config.is_router {
                return Err(WorkflowError::InvalidRouter {
                    node: format!("{:?}", node_config.node_type),
                });
            }

            // Check that all connections point to existing nodes
            for &connected_node in &node_config.connections {
                if !all_node_types.contains(&connected_node) {
                    return Err(WorkflowError::configuration_error(
                        format!("Node {:?} connects to non-existent node {:?}", 
                                node_config.node_type, connected_node),
                        "connections",
                        "workflow_validation",
                        "connections to existing nodes only",
                        Some(format!("invalid_connection_to_{:?}", connected_node)),
                    ));
                }
            }

            // Check that parallel nodes exist
            for &parallel_node in &node_config.parallel_nodes {
                if !all_node_types.contains(&parallel_node) {
                    return Err(WorkflowError::configuration_error(
                        format!("Node {:?} has non-existent parallel node {:?}", 
                                node_config.node_type, parallel_node),
                        "parallel_nodes",
                        "workflow_validation",
                        "parallel nodes must exist in workflow",
                        Some(format!("invalid_parallel_node_{:?}", parallel_node)),
                    ));
                }
            }

            // Check for self-references
            if node_config.connections.contains(&node_config.node_type) {
                return Err(WorkflowError::CycleDetected);
            }

            if node_config.parallel_nodes.contains(&node_config.node_type) {
                return Err(WorkflowError::configuration_error(
                    format!("Node {:?} cannot reference itself in parallel nodes", 
                            node_config.node_type),
                    "parallel_nodes",
                    "workflow_validation",
                    "no self-references in parallel nodes",
                    Some(format!("self_reference_{:?}", node_config.node_type)),
                ));
            }
        }
        Ok(())
    }

    fn validate_complex_cycles(&self) -> Result<(), WorkflowError> {
        // More sophisticated cycle detection including multi-step cycles
        for node_config in &self.schema.nodes {
            if self.has_complex_cycle_from(node_config.node_type)? {
                return Err(WorkflowError::CycleDetected);
            }
        }
        Ok(())
    }

    fn has_complex_cycle_from(&self, start_node: TypeId) -> Result<bool, WorkflowError> {
        let mut visited = HashSet::new();
        let mut path = HashSet::new();
        self.dfs_cycle_check(start_node, &mut visited, &mut path)
    }

    fn dfs_cycle_check(
        &self,
        node: TypeId,
        visited: &mut HashSet<TypeId>,
        path: &mut HashSet<TypeId>,
    ) -> Result<bool, WorkflowError> {
        if path.contains(&node) {
            // We've found a cycle
            return Ok(true);
        }

        if visited.contains(&node) {
            // Already processed this node
            return Ok(false);
        }

        visited.insert(node);
        path.insert(node);

        if let Some(node_config) = self.schema.nodes.iter().find(|nc| nc.node_type == node) {
            // Check all connections
            for &neighbor in &node_config.connections {
                if self.dfs_cycle_check(neighbor, visited, path)? {
                    return Ok(true);
                }
            }
            
            // Check parallel nodes for cycles too
            for &parallel_node in &node_config.parallel_nodes {
                if self.dfs_cycle_check(parallel_node, visited, path)? {
                    return Ok(true);
                }
            }
        }

        path.remove(&node);
        Ok(false)
    }
}
