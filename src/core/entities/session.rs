use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User session entity (Pure Domain Model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: String,
    pub fcm_token: Option<String>,
    pub user_agent: String,
    pub ip_address: String,
    pub auth_exp: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input data for creating/updating a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInput {
    pub user_id: Uuid,
    pub device_id: String,
    pub fcm_token: Option<String>,
    pub user_agent: String,
    pub ip_address: String,
    pub auth_exp: DateTime<Utc>,
}
