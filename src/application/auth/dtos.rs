use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::core::entities::{Gender, FullUserRecord, Session};

use salvo::oapi::ToSchema;

#[derive(Deserialize, ToSchema, Debug)]
pub struct FirebaseLoginRequest {
    pub id_token: String,
    pub fcm_token: Option<String>,
    pub device_id: String,
    pub user_agent: Option<String>,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct FirebaseLoginResponse {
    pub id: Uuid,
    pub firebase_uid: String,
    pub session_id: Uuid,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<chrono::NaiveDate>,
    pub current_auth_provider_slug: String,
    pub current_provider_internal_uid: Option<String>,
    pub is_verified: bool,
    pub token: String,
    pub exp: DateTime<Utc>,
}

pub struct AuthenticationResult {
    pub user: FullUserRecord,
    pub session: Session,
    pub jwt_token: String,
    pub exp: DateTime<Utc>,
    pub current_provider_name: String,
    pub current_provider_internal_uid: Option<String>,
    pub is_verified: bool,
}
