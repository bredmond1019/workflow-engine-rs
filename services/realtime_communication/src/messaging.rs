//! Message routing and broadcasting

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageType {
    ProgressUpdate {
        progress: f32,
        status: String,
        message: String,
    },
    Notification {
        level: String,
        message: String,
    },
    AgentMessage {
        agent_id: String,
        content: String,
    },
    SystemAlert {
        severity: String,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub id: Uuid,
    pub user_id: String,
    pub session_id: String,
    #[serde(flatten)]
    pub message_type: MessageType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}