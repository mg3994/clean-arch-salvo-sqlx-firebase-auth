use chrono::{DateTime, NaiveDate, Utc};
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "gender_enum", rename_all = "snake_case")]
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

/// Core User entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub firebase_uid: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<NaiveDate>,
    pub embedding_dirty: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Full user record with username join
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FullUserRecord {
    pub id: Uuid,
    pub firebase_uid: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<NaiveDate>,
    pub embedding_dirty: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Authentication identity for a user
#[derive(Debug, Clone)]
pub struct AuthIdentity {
    pub provider_slug: String,
    pub provider_uid: String,
    pub verified_at: Option<DateTime<Utc>>,
}

/// Provider type for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderType {
    pub slug: String,
    pub name: String,
}
