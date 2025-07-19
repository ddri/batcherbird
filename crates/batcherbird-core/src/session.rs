use crate::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Session {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            created_at: now,
            updated_at: now,
        }
    }
}

pub struct SessionManager {
    current_session: Option<Session>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            current_session: None,
        }
    }

    pub fn create_session(&mut self, name: String) -> Result<&Session> {
        let session = Session::new(name);
        self.current_session = Some(session);
        Ok(self.current_session.as_ref().unwrap())
    }

    pub fn current_session(&self) -> Option<&Session> {
        self.current_session.as_ref()
    }
}