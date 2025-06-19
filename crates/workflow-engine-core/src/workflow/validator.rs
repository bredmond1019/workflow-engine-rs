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
        self.validate_dag()?;
        self.validate_connections()?;
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
            if !visited.contains(&node_config.node_type) {
                if dfs(
                    node_config.node_type,
                    self.schema,
                    &mut visited,
                    &mut rec_stack,
                ) {
                    return true;
                }
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
        for node_config in &self.schema.nodes {
            if node_config.connections.len() > 1 && !node_config.is_router {
                return Err(WorkflowError::InvalidRouter {
                    node: format!("{:?}", node_config.node_type),
                });
            }
        }
        Ok(())
    }
}
