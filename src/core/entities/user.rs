use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;
use salvo::oapi::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
    NonBinary,
    Transgender,
    Intersex,
    PreferNotToSay,
    Other,
}

/// Core User entity (Pure Domain Model)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub firebase_uid: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub phone_number: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<NaiveDate>,
    pub embedding_dirty: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Full user record with username join (Domain Model)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FullUserRecord {
    pub id: Uuid,
    pub firebase_uid: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub phone_number: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<NaiveDate>,
    pub embedding_dirty: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Authentication identity for a user
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthIdentity {
    pub provider_slug: String,
    pub provider_uid: String,
    pub identifier: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
}
