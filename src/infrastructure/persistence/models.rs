use sqlx::prelude::FromRow;
use sqlx::types::chrono::{DateTime, NaiveDate, Utc};
use sqlx::Type;
use uuid::Uuid;

use crate::core::entities::{User, FullUserRecord, Gender};

#[derive(Debug, Clone, Type)]
#[sqlx(type_name = "gender_enum", rename_all = "snake_case")]
pub enum GenderDb {
    Male,
    Female,
    NonBinary,
    Transgender,
    Intersex,
    PreferNotToSay,
    Other,
}

impl From<Gender> for GenderDb {
    fn from(g: Gender) -> Self {
        match g {
            Gender::Male => Self::Male,
            Gender::Female => Self::Female,
            Gender::NonBinary => Self::NonBinary,
            Gender::Transgender => Self::Transgender,
            Gender::Intersex => Self::Intersex,
            Gender::PreferNotToSay => Self::PreferNotToSay,
            Gender::Other => Self::Other,
        }
    }
}

impl From<GenderDb> for Gender {
    fn from(g: GenderDb) -> Self {
        match g {
            GenderDb::Male => Self::Male,
            GenderDb::Female => Self::Female,
            GenderDb::NonBinary => Self::NonBinary,
            GenderDb::Transgender => Self::Transgender,
            GenderDb::Intersex => Self::Intersex,
            GenderDb::PreferNotToSay => Self::PreferNotToSay,
            GenderDb::Other => Self::Other,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub firebase_uid: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub phone_number: Option<String>,
    pub gender: Option<GenderDb>,
    pub dob: Option<NaiveDate>,
    pub embedding_dirty: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            firebase_uid: row.firebase_uid,
            username: row.username,
            display_name: row.display_name,
            bio: row.bio,
            avatar_url: row.avatar_url,
            phone_number: row.phone_number,
            gender: row.gender.map(Into::into),
            dob: row.dob,
            embedding_dirty: row.embedding_dirty,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct FullUserRecordRow {
    pub id: Uuid,
    pub firebase_uid: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub phone_number: Option<String>,
    pub gender: Option<GenderDb>,
    pub dob: Option<NaiveDate>,
    pub embedding_dirty: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<FullUserRecordRow> for FullUserRecord {
    fn from(row: FullUserRecordRow) -> Self {
        Self {
            id: row.id,
            firebase_uid: row.firebase_uid,
            username: row.username,
            display_name: row.display_name,
            bio: row.bio,
            avatar_url: row.avatar_url,
            phone_number: row.phone_number,
            gender: row.gender.map(Into::into),
            dob: row.dob,
            embedding_dirty: row.embedding_dirty,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct SessionRow {
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

use crate::core::entities::Session;
impl From<SessionRow> for Session {
    fn from(row: SessionRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            device_id: row.device_id,
            fcm_token: row.fcm_token,
            user_agent: row.user_agent,
            ip_address: row.ip_address,
            auth_exp: row.auth_exp,
            revoked_at: row.revoked_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
