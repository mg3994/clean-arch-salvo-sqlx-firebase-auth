use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::core::entities::{User, Gender};
use chrono::NaiveDate;
use salvo::oapi::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub phone_number: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<NaiveDate>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub phone_number: Option<String>,
    pub gender: Option<Option<Gender>>,
    pub dob: Option<NaiveDate>,
}

impl From<User> for UserProfileResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            bio: user.bio,
            avatar_url: user.avatar_url,
            phone_number: user.phone_number,
            gender: Some(user.gender),
            dob: user.dob,
        }
    }
}
