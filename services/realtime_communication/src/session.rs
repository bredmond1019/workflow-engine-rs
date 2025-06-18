//! Session persistence and management

use dashmap::DashMap;
use uuid::Uuid;

pub struct SessionManager {
    sessions: DashMap<Uuid, SessionData>,
}

pub struct SessionData {
    pub user_id: String,
    pub connection_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    pub fn create_session(&self, user_id: String, connection_id: Uuid) -> Uuid {
        let session_id = Uuid::new_v4();
        let session = SessionData {
            user_id,
            connection_id,
            created_at: chrono::Utc::now(),
        };
        self.sessions.insert(session_id, session);
        session_id
    }
}