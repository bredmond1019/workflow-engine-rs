use std::fmt;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    core::{error::WorkflowError, task::TaskContext},
    db::schema::events,
};

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Insertable, Selectable, Identifiable, Default,
)]
#[diesel(table_name = events)]
pub struct Event {
    pub id: Uuid,
    pub workflow_type: String,
    // Data is a JSON object
    pub data: Value,
    // Task Context is a JSON object
    pub task_context: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Event {
    pub fn store(&self, conn: &mut PgConnection) -> Result<(), diesel::result::Error> {
        diesel::insert_into(events::table)
            .values(self)
            .execute(conn)?;
        Ok(())
    }

    pub fn update_task_context(&mut self, task_context: &TaskContext) -> Result<(), WorkflowError> {
        self.task_context =
            serde_json::to_value(task_context).map_err(|e| WorkflowError::SerializationError {
                message: format!("Failed to serialize task context: {}", e),
            })?;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn get_typed_data<T: for<'de> Deserialize<'de>>(&self) -> Result<T, WorkflowError> {
        serde_json::from_value(self.data.clone()).map_err(|e| WorkflowError::DeserializationError {
            message: format!("Failed to deserialize event data: {}", e),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEvent {
    pub data: Value,
    pub workflow_type: String,
    pub task_context: Value,
}

impl NewEvent {
    pub fn new(data: Value, workflow_type: String, task_context: Value) -> Event {
        let now = Utc::now();
        Event {
            id: Uuid::new_v4(),
            workflow_type,
            data,
            task_context,
            created_at: now,
            updated_at: now,
        }
    }
}
